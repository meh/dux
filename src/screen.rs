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

use xcbu;

use {Display, error};

/// Manages luminances and screen content through the MIT-SHM extension.
pub struct Screen {
	display: Arc<Display>,

	image:      xcbu::image::shm::Image,
	luminances: Vec<f32>,
	luminance:  u64,
}

const PRECISION: f32 = 1_000_000.0;

impl Screen {
	/// Create a new screen holder.
	pub fn open(display: Arc<Display>) -> error::Result<Screen> {
		// Create an image in shared memory as big as the display.
		let image = xcbu::image::shm::create(&display, 24, display.width as u16, display.height as u16)?;

		// Set up the luminances vector, again as big as the display.
		let luminances = vec![0.0; (display.width * display.height) as usize];

		Ok(Screen {
			display:    display,
			image:      image,
			luminances: luminances,
			luminance:  0,
		})
	}

	/// Get the given screen section and update the luminance values.
	pub fn refresh(&mut self, x: u32, y: u32, width: u32, height: u32) -> error::Result<()> {
		// Note that this will resize the image to fit the section, but that
		// doesn't matter because it will never be bigger than the screen.
		xcbu::image::shm::area(&self.display.clone(), self.display.root, &mut self.image,
			x as i16, y as i16, width as u16, height as u16, !0)?;

		// Update the pixel values relative to the section.
		for xx in x .. width {
			for yy in y .. height {
				let px = self.image.get(xx - x, yy - y);
				self.put(xx, yy, px);
			}
		}

		Ok(())
	}

	/// Puts a pixel at the given coordinates, updating the total luminance.
	pub fn put(&mut self, x: u32, y: u32, pixel: u32) -> f32 {
		// Extract the RGB channels and normalize them to `0.0` - `1.0`.
		let r = ((pixel & 0xff0000) >> 16) as f32 / 255.0;
		let g = ((pixel & 0x00ff00) >> 8) as f32 / 255.0;
		let b = (pixel & 0x0000ff) as f32 / 255.0;

		let r = if r > 0.4045 { ((r + 0.055) / 1.055).powf(2.4) } else { r / 12.92 };
		let g = if g > 0.4045 { ((g + 0.055) / 1.055).powf(2.4) } else { g / 12.92 };
		let b = if b > 0.4045 { ((b + 0.055) / 1.055).powf(2.4) } else { b / 12.92 };

		let l = ((r * 0.2126) + (g * 0.7152) + (b * 0.0722)) / 1.0;
		let l = if l > 0.008856 { l.powf(1.0 / 3.0) } else { (l * 7.787) + (16.0 / 116.0) };
		let l = (l * 116.0) - 16.0;

		// The index within the luminance vector based on the position.
		let i = (y * self.display.width + x) as usize;

		// Update the total luminance in place, we use an `u64` to contain the
		// total to avoid incremental precision errors because of the repeated
		// operations.
		//
		// The actual value is clamped to a constant precision and converted to an
		// `u64` value.
		self.luminance -= (self.luminances[i].powi(2) * PRECISION) as u64;
		self.luminance += (l.powi(2) * PRECISION) as u64;

		// Save the new luminance so it can be restored when this pixel is changed.
		self.luminances[i] = l;

		l
	}

	/// Get the square root of the total luminance.
	pub fn luminance(&self) -> f32 {
		((self.luminance as f32 / PRECISION)
			/ (self.display.width * self.display.height) as f32).sqrt()
	}
}
