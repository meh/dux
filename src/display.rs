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

use error;

pub struct Display {
	pub connection: xcbu::ewmh::Connection,
	pub screen:     i32,
	pub root:       xcb::Window,

	pub width:  u32,
	pub height: u32,

	pub randr:  xcb::QueryExtensionData,
	pub shm:    xcb::QueryExtensionData,
	pub damage: xcb::QueryExtensionData,
}

unsafe impl Send for Display { }
unsafe impl Sync for Display { }

impl Display {
	pub fn open() -> error::Result<Self> {
		let (connection, screen)  = xcb::Connection::connect(None)?;
		let connection            = xcbu::ewmh::Connection::connect(connection).map_err(|(e, _)| e)?;
		let (root, width, height) = {
			let screen = connection.get_setup().roots().nth(screen as usize).unwrap();

			(screen.root(), screen.width_in_pixels(), screen.height_in_pixels())
		};

		let randr = {
			let extension = connection.get_extension_data(xcb::randr::id()).ok_or(error::Error::Unsupported)?;
			let version   = xcb::randr::query_version(&connection, 1, 2).get_reply()?;

			if version.major_version() != 1 || version.minor_version() < 2 {
				return Err(error::Error::Unsupported);
			}

			extension
		};

		let shm = {
			let extension = connection.get_extension_data(xcb::shm::id()).ok_or(error::Error::Unsupported)?;
			let version   = xcb::shm::query_version(&connection).get_reply()?;

			if version.major_version() != 1 || version.minor_version() < 1 {
				return Err(error::Error::Unsupported);
			}

			extension
		};

		let damage = {
			let extension = connection.get_extension_data(xcb::damage::id()).ok_or(error::Error::Unsupported)?;
			let version   = xcb::damage::query_version(&connection, 1, 1).get_reply()?;

			if version.major_version() != 1 || version.minor_version() < 1 {
				return Err(error::Error::Unsupported);
			}

			extension
		};

		Ok(Display {
			connection: connection,
			screen:     screen,
			root:       root,

			width:  width as u32,
			height: height as u32,

			randr:  randr,
			shm:    shm,
			damage: damage,
		})
	}
}

impl Deref for Display {
	type Target = xcbu::ewmh::Connection;

	fn deref(&self) -> &Self::Target {
		&self.connection
	}
}
