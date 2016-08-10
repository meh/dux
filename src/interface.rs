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
use std::sync::mpsc::{Receiver, channel, sync_channel};
use std::ops::Deref;

use dbus;

use {error, backlight};

/// DBus interface handler.
pub struct Interface {
	receiver: Receiver<Event>,
}

#[derive(Debug)]
pub enum Event {
	Mode(Mode),
	Profile(String),
	Brightness(f32),
	Save,
	Stop,

	ScreenSaver(bool),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Mode {
	Manual,
	Desktop,
	Window,
	Luminance,
	Time,
}

impl Default for Mode {
	fn default() -> Self {
		Mode::Luminance
	}
}

impl Mode {
	pub fn parse<T: AsRef<str>>(value: T) -> Option<Mode> {
		match value.as_ref() {
			"manual"    => Some(Mode::Manual),
			"desktop"   => Some(Mode::Desktop),
			"window"    => Some(Mode::Window),
			"luminance" => Some(Mode::Luminance),
			"time"      => Some(Mode::Time),
			_           => None,
		}
	}
}

impl Interface {
	/// Change the adaptive mode.
	pub fn mode<T: Into<String>>(value: T) -> error::Result<()> {
		dbus::Connection::get_private(dbus::BusType::Session)?
			.send(dbus::Message::new_method_call(
				"meh.rust.Backlight",
				"/meh/rust/Backlight",
				"meh.rust.Backlight",
				"Mode")?
					.append1(value.into()))?;

		Ok(())
	}

	/// Change the current profile.
	pub fn profile<T: Into<String>>(value: T) -> error::Result<()> {
		dbus::Connection::get_private(dbus::BusType::Session)?
			.send(dbus::Message::new_method_call(
				"meh.rust.Backlight",
				"/meh/rust/Backlight",
				"meh.rust.Backlight",
				"Profile")?
					.append1(value.into()))?;

		Ok(())
	}

	/// Send a brightness change.
	pub fn brightness(value: f32) -> error::Result<()> {
		dbus::Connection::get_private(dbus::BusType::Session)?
			.send(dbus::Message::new_method_call(
				"meh.rust.Backlight",
				"/meh/rust/Backlight",
				"meh.rust.Backlight",
				"Brightness")?
					.append1(backlight::clamp(value) as f64))?;

		Ok(())
	}

	/// Send a save event.
	pub fn save() -> error::Result<()> {
		dbus::Connection::get_private(dbus::BusType::Session)?
			.send(dbus::Message::new_method_call(
				"meh.rust.Backlight",
				"/meh/rust/Backlight",
				"meh.rust.Backlight",
				"Save")?)?;

		Ok(())
	}

	/// Send a stop event.
	pub fn stop() -> error::Result<()> {
		dbus::Connection::get_private(dbus::BusType::Session)?
			.send(dbus::Message::new_method_call(
				"meh.rust.Backlight",
				"/meh/rust/Backlight",
				"meh.rust.Backlight",
				"Stop")?)?;

		Ok(())
	}

	/// Spawn the server.
	pub fn spawn() -> error::Result<Self> {
		let (sender, receiver)     = sync_channel(1);
		let (g_sender, g_receiver) = channel::<error::Result<()>>();

		macro_rules! dbus {
			(connect) => (
				match dbus::Connection::get_private(dbus::BusType::Session) {
					Ok(value) => {
						value
					}

					Err(error) => {
						g_sender.send(Err(error.into())).unwrap();
						return;
					}
				}
			);

			(register $conn:expr, $name:expr) => (
				match $conn.register_name($name, dbus::NameFlag::DoNotQueue as u32) {
					Ok(dbus::RequestNameReply::Exists) => {
						g_sender.send(Err(error::DBus::AlreadyRegistered.into())).unwrap();
						return;
					}

					Err(error) => {
						g_sender.send(Err(error.into())).unwrap();
						return;
					}

					Ok(value) => {
						value
					}
				}
			);

			(watch $conn:expr, $filter:expr) => (
				if let Err(error) =  $conn.add_match($filter) {
					g_sender.send(Err(error.into())).unwrap();
					return;
				}
			);

			(ready) => (
				g_sender.send(Ok(())).unwrap();
			);

			(check) => (
				g_receiver.recv().unwrap()
			)
		}

		thread::spawn(move || {
			let c = dbus!(connect);
			let f = dbus::tree::Factory::new_fn();

			dbus!(register c, "meh.rust.Backlight");
			dbus!(watch c, "interface='org.gnome.ScreenSaver',member='ActiveChanged'");
			dbus!(ready);

			let tree = f.tree().add(f.object_path("/meh/rust/Backlight").introspectable().add(f.interface("meh.rust.Backlight")
				.add_m(f.method("Mode", |m, _, _| {
					if let Some(value) = m.get1::<String>().and_then(Mode::parse) {
						sender.send(Event::Mode(value)).unwrap();

						Ok(vec![m.method_return()])
					}
					else {
						Err(dbus::tree::MethodErr::no_arg())
					}
				}).inarg::<String, _>("mode"))

				.add_m(f.method("Profile", |m, _, _| {
					if let Some(value) = m.get1::<String>() {
						sender.send(Event::Profile(value)).unwrap();

						Ok(vec![m.method_return()])
					}
					else {
						Err(dbus::tree::MethodErr::no_arg())
					}
				}).inarg::<String, _>("profile"))

				.add_m(f.method("Brightness", |m, _, _| {
					if let Some(value) = m.get1::<f64>() {
						sender.send(Event::Brightness(value as f32)).unwrap();

						Ok(vec![m.method_return()])
					}
					else {
						Err(dbus::tree::MethodErr::no_arg())
					}
				}).inarg::<f64, _>("value"))

				.add_m(f.method("Save", |m, _, _| {
					sender.send(Event::Save).unwrap();

					Ok(vec![m.method_return()])
				}))

				.add_m(f.method("Stop", |m, _, _| {
					sender.send(Event::Stop).unwrap();

					Ok(vec![m.method_return()])
				}))));

			tree.set_registered(&c, true).unwrap();
			for item in tree.run(&c, c.iter(1_000_000)) {
				if let dbus::ConnectionItem::Signal(m) = item {
					match (&*m.interface().unwrap(), &*m.member().unwrap()) {
						("org.gnome.ScreenSaver", "ActiveChanged") => {
							if let Some(status) = m.get1() {
								sender.send(Event::ScreenSaver(status)).unwrap();
							}
						}

						_ => ()
					}
				}
			}
		});

		dbus!(check)?;

		Ok(Interface {
			receiver: receiver,
		})
	}
}

impl Deref for Interface {
	type Target = Receiver<Event>;

	fn deref(&self) -> &Self::Target {
		&self.receiver
	}
}
