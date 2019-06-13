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
use std::time::Instant;
use std::cmp;

use xcb;
use xcbu;

use crate::{Display, error};

/// Manages luminances and screen content through the MIT-SHM extension.
pub struct Screen {
	display: Arc<Display>,
	image:   xcbu::image::shm::Image,

	width:  u32,
	height: u32,

	luminances: Vec<f32>,
	luminance:  u64,

	cache: Vec<(u32, u32, u32, u32)>,
	rated: Option<Instant>,
}

const PRECISION: f32 = 1_000_000.0;

impl Screen {
	/// Create a new screen holder.
	pub fn open(display: Arc<Display>, width: u32, height: u32) -> error::Result<Screen> {
		// Create an image in shared memory as big as the display.
		let image = xcbu::image::shm::create(&display, 24, width as u16, height as u16)?;

		// Set up the luminances vector, again as big as the display.
		let luminances = vec![0.0; (width * height) as usize];

		Ok(Screen {
			display: display,
			image:   image,

			width:  width,
			height: height,

			luminances: luminances,
			luminance:  0,

			cache: Vec::new(),
			rated: None,
		})
	}

	/// Resize the screen.
	pub fn resize(&mut self, width: u32, height: u32) -> error::Result<()> {
		// Create a new image only if the new size is bigger than the actual size.
		if self.image.actual_width() * self.image.actual_height() < (width * height) as u16 {
			self.image = xcbu::image::shm::create(&self.display, 24, width as u16, height as u16)?;
		}
		else {
			self.image.resize(width as u16, height as u16);
		}

		self.width  = width;
		self.height = height;

		// Reset the luminance values.
		self.luminances.resize((width * height) as usize, 0.0);
		self.luminance = 0;

		// This gets optimized to a memset.
		for item in &mut self.luminances {
			*item = 0.0;
		}

		// Update the whole screen.
		self.refresh(0, 0, width, height)
	}

	/// Flush any cached damages.
	pub fn flush(&mut self) -> error::Result<()> {
		if self.rated.is_none() || self.cache.is_empty() {
			return Ok(());
		}

		let (x, y, w, h) = self.cache.drain(..).fold((u32::max_value(), u32::max_value(), 0, 0),
			|(xmin, ymin, xmax, ymax), (x, y, w, h)| {
				(cmp::min(xmin, x), cmp::min(ymin, y), cmp::max(xmax, x + w), cmp::max(ymax, y + h))
			});

		self.rated = None;
		self.refresh(x, y, w - x, h - y)
	}

	/// Mark a screen area as damaged.
	pub fn damage(&mut self, rect: xcb::Rectangle, threshold: u64) -> error::Result<bool> {
		let x = rect.x() as u32;
		let y = rect.y() as u32;
		let w = rect.width() as u32;
		let h = rect.height() as u32;

		if self.rated.is_some() && (w * h) as u64 >= threshold {
			self.cache.push((x, y, w, h));

			Ok(false)
		}
		else if (w * h) as u64 >= threshold {
			self.rated = Some(Instant::now());
			self.refresh(x, y, w, h)?;

			Ok(false)
		}
		else {
			self.refresh(x, y, w, h)?;

			Ok(true)
		}
	}

	/// Get the given screen section and update the luminance values.
	pub fn refresh(&mut self, x: u32, y: u32, width: u32, height: u32) -> error::Result<()> {
		// Note that this will resize the image to fit the section, but that
		// doesn't matter because it will never be bigger than the screen.
		xcbu::image::shm::area(&self.display, self.display.root(), &mut self.image,
			x as i16, y as i16, width as u16, height as u16, !0)?;

		// Update the pixel values relative to the section.
		for xx in x .. width {
			for yy in y .. height {
				let rgb = {
					let data   = self.image.data();
					let offset = (((xx - x) * 4) + ((yy - y) * width * 4)) as usize;

					(data[offset], data[offset + 1], data[offset + 2])
				};

				self.put(xx, yy, rgb);
			}
		}

		Ok(())
	}

	/// Puts a pixel at the given coordinates, updating the total luminance.
	pub fn put(&mut self, x: u32, y: u32, (r, g, b): (u8, u8, u8)) -> f32 {
		// Extract the RGB channels and normalize them to `0.0` - `1.0`.
		let r = r as f32 / 255.0;
		let g = g as f32 / 255.0;
		let b = b as f32 / 255.0;

		// Calculate the perceived luminance.
		let l = (r * 0.299) + (g * 0.587) + (b * 0.114);

		// The index within the luminance vector based on the position.
		let i = (x + (y * self.width)) as usize;

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

	/// Get the RMS luminance.
	pub fn luminance(&self) -> f32 {
		((self.luminance as f32 / PRECISION) / (self.width * self.height) as f32).sqrt()
	}
}
