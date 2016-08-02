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

pub trait Backlight {
	fn get(&mut self) -> f32;
	fn set(&mut self, value: f32);
}

macro_rules! try {
	($body:expr) => (
		if let Some(value) = $body {
			value
		}
		else {
			return None;
		}
	);
}

mod randr;
mod sys;

pub fn open(connection: Rc<xcb::Connection>, screen: i32) -> Option<Box<Backlight>> {
	if let Some(backlight) = randr::Backlight::open(connection.clone(), screen) {
		Some(Box::new(backlight))
	}
	else if let Some(backlight) = sys::Backlight::open() {
		Some(Box::new(backlight))
	}
	else {
		None
	}
}
