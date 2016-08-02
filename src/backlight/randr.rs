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

use std::rc::Rc;

use xcb;
use byteorder::{NativeEndian, ReadBytesExt};

pub struct Backlight {
	connection: Rc<xcb::Connection>,
	output:     xcb::randr::Output,
	atom:       xcb::Atom,
	range:      (i32, i32),
}

impl Backlight {
	pub fn open(connection: Rc<xcb::Connection>, screen: i32) -> Option<Self> {
		fn find(c: &xcb::Connection, screen: i32) -> Option<(xcb::randr::Output, xcb::Atom)> {
			let current = try!(xcb::intern_atom(c, true, "Backlight").get_reply().ok()
				.and_then(|r| if r.atom() != xcb::ATOM_NONE { Some(r.atom()) } else { None }));

			let legacy = try!(xcb::intern_atom(c, true, "BACKLIGHT").get_reply().ok()
				.and_then(|r| if r.atom() != xcb::ATOM_NONE { Some(r.atom()) } else { None }));

			let screen    = c.get_setup().roots().nth(screen as usize).unwrap();
			let resources = try!(xcb::randr::get_screen_resources_current(c, screen.root()).get_reply().ok());

			for &id in resources.outputs() {
				let reply = if let Ok(r) = xcb::randr::get_output_property(c, id, current, xcb::ATOM_NONE, 0, 4, false, false).get_reply() {
					Some((r, current))
				}
				else if let Ok(r) = xcb::randr::get_output_property(c, id, legacy, xcb::ATOM_NONE, 0, 4, false, false).get_reply() {
					Some((r, legacy))
				}
				else {
					None
				};

				if let Some((reply, atom)) = reply {
					if reply.type_() == xcb::ATOM_INTEGER && reply.num_items() == 1 && reply.format() == 32 {
						return Some((id, atom));
					}
				}
			}

			None
		}

		try!(connection.get_extension_data(xcb::randr::id()));
		let reply = try!(xcb::randr::query_version(&connection, 1, 2).get_reply().ok());

		if reply.major_version() != 1 || reply.minor_version() < 2 {
			return None;
		}

		let (output, atom) = try!(find(&connection, screen));
		let range          = {
			let reply = try!(xcb::randr::query_output_property(&connection, output, atom).get_reply().ok());
			(reply.validValues()[0], reply.validValues()[1])
		};

		Some(Backlight {
			connection: connection,
			output:     output,
			atom:       atom,
			range:      range,
		})
	}
}

impl super::Backlight for Backlight {
	fn get(&mut self) -> f32 {
		let raw = xcb::randr::get_output_property(&self.connection, self.output, self.atom, xcb::ATOM_NONE, 0, 4, false, false)
			.get_reply().unwrap().data().read_i32::<NativeEndian>().unwrap();

		((raw - self.range.0) * 100) as f32 / (self.range.1 - self.range.0) as f32
	}

	fn set(&mut self, value: f32) {
		xcb::randr::change_output_property(&self.connection, self.output, self.atom, xcb::ATOM_INTEGER, 32, xcb::PROP_MODE_REPLACE as u8,
			&[(self.range.0 + (value * (self.range.1 - self.range.0) as f32 / 100.0) as i32)]);
		self.connection.flush();
	}
}
