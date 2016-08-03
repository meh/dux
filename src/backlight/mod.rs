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

use error;

pub trait Backlight {
	fn get(&mut self) -> error::Result<f32>;
	fn set(&mut self, value: f32) -> error::Result<()>;
}

mod randr;
mod sys;

pub fn open(connection: Arc<xcb::Connection>, screen: i32) -> error::Result<Box<Backlight>> {
	if let Ok(backlight) = randr::Backlight::open(connection.clone(), screen) {
		Ok(Box::new(backlight))
	}
	else if let Ok(backlight) = sys::Backlight::open() {
		Ok(Box::new(backlight))
	}
	else {
		Err(error::Error::Unsupported)
	}
}
