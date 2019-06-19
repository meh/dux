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

use std::sync::Arc;

use xcb;
use byteorder::{NativeEndian, ReadBytesExt};

use crate::{Display, error};

pub struct Backlight {
	display: Arc<Display>,
	output:  xcb::randr::Output,
	atom:    xcb::Atom,
	range:   (i32, i32),
}

impl Backlight {
	pub fn open(display: Arc<Display>) -> error::Result<Self> {
		fn find(display: &Display) -> error::Result<(xcb::randr::Output, xcb::Atom)> {
			let current = xcb::intern_atom(display, true, "Backlight").get_reply().ok()
				.and_then(|r| if r.atom() != xcb::ATOM_NONE { Some(r.atom()) } else { None })
				.ok_or(error::Error::Unsupported)?;

			let legacy = xcb::intern_atom(display, true, "BACKLIGHT").get_reply().ok()
				.and_then(|r| if r.atom() != xcb::ATOM_NONE { Some(r.atom()) } else { None })
				.ok_or(error::Error::Unsupported)?;

			for &id in xcb::randr::get_screen_resources_current(display, display.root()).get_reply()?.outputs() {
				let reply = if let Ok(r) = xcb::randr::get_output_property(display, id, current, xcb::ATOM_NONE, 0, 4, false, false).get_reply() {
					Some((r, current))
				}
				else if let Ok(r) = xcb::randr::get_output_property(display, id, legacy, xcb::ATOM_NONE, 0, 4, false, false).get_reply() {
					Some((r, legacy))
				}
				else {
					None
				};

				if let Some((reply, atom)) = reply {
					if reply.type_() == xcb::ATOM_INTEGER && reply.num_items() == 1 && reply.format() == 32 {
						return Ok((id, atom));
					}
				}
			}

			Err(error::Error::Unsupported)
		}

		let (output, atom) = find(&display)?;
		let range          = xcb::randr::query_output_property(&display, output, atom).get_reply().map(|reply|
			(reply.valid_values()[0], reply.valid_values()[1]))?;

		Ok(Backlight { display, output, atom, range })
	}
}

impl super::Backlight for Backlight {
	fn range(&self) -> (u32, u32) {
		(self.range.0 as u32, self.range.1 as u32)
	}

	fn get(&mut self) -> error::Result<f32> {
		let raw = xcb::randr::get_output_property(&self.display, self.output, self.atom, xcb::ATOM_NONE, 0, 4, false, false)
			.get_reply()?.data().read_i32::<NativeEndian>()?;

		Ok(((raw - self.range.0) * 100) as f32 / (self.range.1 - self.range.0) as f32)
	}

	fn set(&mut self, value: f32) -> error::Result<()> {
		xcb::randr::change_output_property(&self.display, self.output, self.atom, xcb::ATOM_INTEGER, 32, xcb::PROP_MODE_REPLACE as u8,
			&[(self.range.0 + (super::clamp(value) * (self.range.1 - self.range.0) as f32 / 100.0) as i32)]);

		self.display.flush();

		Ok(())
	}
}
