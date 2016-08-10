// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of dux.
//
// dux is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// dux is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with dux.  If not, see <http://www.gnu.org/licenses/>.

use std::thread;
use std::sync::mpsc::{Receiver, sync_channel};
use std::sync::Arc;
use std::ops::Deref;

use xcb;

use {Display, error};

/// Handles events from a `Display` and sends the appropriate ones.
pub struct Observer {
	receiver: Receiver<Event>,
}

/// Events from the `Display`.
pub enum Event {
	/// A window is being shown.
	Show(xcb::Window),

	/// A window is being hidden.
	Hide(xcb::Window),

	/// A window's position/size has changed.
	Change(xcb::Window),

	/// Some screen area changed.
	Damage(xcb::Rectangle),

	/// The active window changed.
	Active(Option<xcb::Window>),

	/// The current desktop changed.
	Desktop(i32),

	/// The screen has been resized/rotated.
	Resize(u32, u32),
}

impl Observer {
	/// Get the current desktop.
	pub fn desktop(display: &Display) -> error::Result<i32> {
		xcb::get_property(display, false, display.root(), display.CURRENT_DESKTOP(), xcb::ATOM_CARDINAL, 0, 1)
			.get_reply()?.value::<i32>().get(0).cloned().ok_or(error::Error::Unsupported)
	}

	/// Get the currently active window, if any.
	pub fn window(display: &Display) -> error::Result<Option<xcb::Window>> {
		let id = xcb::get_property(display, false, display.root(), display.ACTIVE_WINDOW(), xcb::ATOM_WINDOW, 0, 1)
			.get_reply()?.value::<xcb::Window>().get(0).cloned().ok_or(error::Error::Unsupported)?;

		if id == 0 {
			Ok(None)
		}
		else {
			Ok(Some(id))
		}
	}

	/// Spawn the observer on the given `Display`.
	pub fn spawn(display: Arc<Display>) -> error::Result<Self> {
		let (sender, receiver) = sync_channel(1);

		// Listen for map/unmap and configure events.
		xcb::change_window_attributes_checked(&display, display.root(), &[
			(xcb::CW_EVENT_MASK,
				xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY |
				xcb::EVENT_MASK_PROPERTY_CHANGE)]).request_check()?;

		// Listen for damage areas, if the other report levels worked it would be
		// nice, but alas, we're gonna get spammed by damages.
		let damage = display.generate_id();
		xcb::damage::create_checked(&display, damage, display.root(), xcb::damage::REPORT_LEVEL_RAW_RECTANGLES as u8)
			.request_check()?;

		thread::spawn(move || {
			// Send the currently active desktop if present.
			if let Ok(id) = Observer::desktop(&display) {
				sender.send(Event::Desktop(id)).unwrap();
			}

			// Send the currently active window if present.
			if let Ok(id) = Observer::window(&display) {
				sender.send(Event::Active(id)).unwrap();
			}

			while let Some(event) = display.wait_for_event() {
				match event.response_type() {
					xcb::MAP_NOTIFY => {
						let event = xcb::cast_event(&event): &xcb::MapNotifyEvent;

						sender.send(Event::Show(event.window())).unwrap();
					}

					xcb::UNMAP_NOTIFY => {
						let event = xcb::cast_event(&event): &xcb::UnmapNotifyEvent;

						sender.send(Event::Hide(event.window())).unwrap();
					}

					xcb::CONFIGURE_NOTIFY => {
						let event = xcb::cast_event(&event): &xcb::ConfigureNotifyEvent;

						sender.send(Event::Change(event.window())).unwrap();
					}

					xcb::PROPERTY_NOTIFY => {
						let event = xcb::cast_event(&event): &xcb::PropertyNotifyEvent;

						match event.atom() {
							prop if prop == display.CURRENT_DESKTOP() && event.state() == xcb::PROPERTY_NEW_VALUE as u8 => {
								if let Ok(id) = Observer::desktop(&display) {
									sender.send(Event::Desktop(id)).unwrap();
								}
							}

							prop if prop == display.ACTIVE_WINDOW() && event.state() == xcb::PROPERTY_NEW_VALUE as u8 => {
								if let Ok(id) = Observer::window(&display) {
									sender.send(Event::Active(id)).unwrap();
								}
							}

							_ => ()
						}
					}

					// Handle damaged rectangles.
					e if e == display.damage().first_event() => {
						let event = xcb::cast_event(&event): &xcb::damage::NotifyEvent;
						sender.send(Event::Damage(event.area())).unwrap();

						// Mark the damage region as handled.
						xcb::damage::subtract(&display, damage, xcb::xfixes::REGION_NONE, xcb::xfixes::REGION_NONE);
						display.flush();
					}

					// Handle screen changes.
					e if e == display.randr().first_event() + xcb::randr::SCREEN_CHANGE_NOTIFY => {
						let event = xcb::cast_event(&event): &xcb::randr::ScreenChangeNotifyEvent;

						if event.root() == display.root() {
							sender.send(Event::Resize(event.width() as u32, event.height() as u32)).unwrap();
						}
					}

					_ => ()
				}
			}
		});

		Ok(Observer {
			receiver: receiver,
		})
	}
}

impl Deref for Observer {
	type Target = Receiver<Event>;

	fn deref(&self) -> &Self::Target {
		&self.receiver
	}
}
