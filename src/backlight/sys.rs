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

use std::fs::{self, File};
use std::path::PathBuf;
use std::io::{Write, Read};

pub struct Backlight {
	path: PathBuf,
	max:  u32,
}

impl Backlight {
	pub fn open() -> Option<Self> {
		let root = try!(try!(try!(fs::read_dir("/sys/class/backlight").ok()).next()).ok()).path();
		let max  = {
			let mut file    = try!(File::open(root.join("max_brightness")).ok());
			let mut content = String::new();

			try!(file.read_to_string(&mut content).ok());
			try!(content.trim().parse::<u32>().ok())
		};

		Some(Backlight {
			path: root.join("brightness"),
			max:  max,
		})
	}
}

impl super::Backlight for Backlight {
	fn get(&mut self) -> f32 {
		let mut file    = File::open(&self.path).unwrap();
		let mut content = String::new();

		file.read_to_string(&mut content).unwrap();
		content.trim().parse::<f32>().unwrap() * 100.0 / self.max as f32
	}

	fn set(&mut self, value: f32) {
		let mut file = File::create(&self.path).unwrap();
		write!(&mut file, "{}", ((value * self.max as f32) / 100.0).round() as u32).unwrap();
	}
}
