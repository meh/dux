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

use error;

pub struct Backlight {
	path: PathBuf,
	max:  u32,
}

impl Backlight {
	pub fn open() -> error::Result<Self> {
		let root = fs::read_dir("/sys/class/backlight")?.next().ok_or(error::Error::Unsupported)??.path();
		let max  = {
			let mut file    = File::open(root.join("max_brightness"))?;
			let mut content = String::new();

			file.read_to_string(&mut content)?;
			content.trim().parse::<u32>().or(Err(error::Error::Unsupported))?
		};

		Ok(Backlight {
			path: root.join("brightness"),
			max:  max,
		})
	}
}

impl super::Backlight for Backlight {
	fn get(&mut self) -> error::Result<f32> {
		let mut file    = File::open(&self.path)?;
		let mut content = String::new();

		file.read_to_string(&mut content)?;
		Ok(content.trim().parse::<f32>().or(Err(error::Error::Unsupported))?
			* 100.0 / self.max as f32)
	}

	fn set(&mut self, value: f32) -> error::Result<()> {
		let mut file = File::create(&self.path)?;
		write!(&mut file, "{}", ((value * self.max as f32) / 100.0).round() as u32)?;

		Ok(())
	}
}
