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

use std::ops::Deref;

use xcb;
use xcbu;

use crate::error;

/// Handles the X11 display.
pub struct Display {
	connection: xcbu::ewmh::Connection,
	screen:     i32,
	root:       xcb::Window,
}

impl Display {
	/// Open the default display.
	pub fn open() -> error::Result<Self> {
		let (connection, screen) = xcb::Connection::connect(None)?;
		let connection           = xcbu::ewmh::Connection::connect(connection).map_err(|(e, _)| e)?;
		let root                 = connection.get_setup().roots().nth(screen as usize).unwrap().root();

		// Randr is used for the backlight and screen configuration changes events.
		{
			let version = xcb::randr::query_version(&connection, 1, 2).get_reply()?;

			if version.major_version() != 1 || version.minor_version() < 2 {
				return Err(error::Error::Unsupported);
			}

			xcb::randr::select_input_checked(&connection, root, xcb::randr::NOTIFY_MASK_SCREEN_CHANGE as u16)
				.request_check()?;
		}

		// MIT-SHM is used to fetch screen contents.
		{
			let version = xcb::shm::query_version(&connection).get_reply()?;

			if version.major_version() != 1 || version.minor_version() < 1 {
				return Err(error::Error::Unsupported);
			}
		}

		// DAMAGE is used to get screen content changes.
		{
			let version  = xcb::damage::query_version(&connection, 1, 1).get_reply()?;

			if version.major_version() != 1 || version.minor_version() < 1 {
				return Err(error::Error::Unsupported);
			}
		}

		Ok(Display {
			connection: connection,
			screen:     screen,
			root:       root,
		})
	}

	/// Get the default screen.
	pub fn screen(&self) -> i32 {
		self.screen
	}

	/// Get the root window for the default screen.
	pub fn root(&self) -> xcb::Window {
		self.root
	}

	/// Get the default screen width.
	pub fn width(&self) -> u32 {
		self.get_setup().roots().nth(self.screen as usize).unwrap().width_in_pixels() as u32
	}

	/// Get the default screen height.
	pub fn height(&self) -> u32 {
		self.get_setup().roots().nth(self.screen as usize).unwrap().height_in_pixels() as u32
	}

	/// Get the XRandr extension details.
	pub fn randr(&self) -> xcb::QueryExtensionData {
		self.connection.get_extension_data(xcb::randr::id()).unwrap()
	}

	/// Get the MIT-SHM extension details.
	pub fn shm(&self) -> xcb::QueryExtensionData {
		self.connection.get_extension_data(xcb::shm::id()).unwrap()
	}

	/// Get the DAMAGE extension details.
	pub fn damage(&self) -> xcb::QueryExtensionData {
		self.connection.get_extension_data(xcb::damage::id()).unwrap()
	}
}

impl Deref for Display {
	type Target = xcbu::ewmh::Connection;

	fn deref(&self) -> &Self::Target {
		&self.connection
	}
}
