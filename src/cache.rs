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

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xdg;
use json::{self, JsonValue, object, array};
use chrono::{self, Timelike};
use xcb;
use xcbu;

use crate::{Display, error};

/// An in memory cache persisted to disk for settings.
///
/// It supports multiple profiles and takes care of saving the brightness
/// values appropriately for each `Mode`.
pub struct Cache {
	display: Arc<Display>,
	data:    JsonValue,
	path:    PathBuf,
	profile: String,
}

/// Supported modes.
pub enum Mode {
	Manual,
	Desktop(i32),
	Window(Option<xcb::Window>),
	Luminance(f32),
	Time(chrono::DateTime<chrono::Local>),
}

impl Cache {
	/// Open the cache at the given path.
	pub fn open<T: AsRef<Path>>(display: Arc<Display>, path: Option<T>) -> error::Result<Self> {
		// If no path was given we use the XDG standard places.
		let path = if let Some(path) = path {
			path.as_ref().into()
		}
		else {
			xdg::BaseDirectories::with_prefix("dux").unwrap()
				.place_config_file("cache.json").unwrap()
		};

		// Load the contents if the file exists.
		let mut data = if path.exists() {
			let mut file    = File::open(&path)?;
			let mut content = String::new();
			file.read_to_string(&mut content)?;

			json::parse(&content).unwrap_or_else(|_| object!{})
		}
		else {
			object!{}
		};

		// Make sure it's set up with basic keys.
		if data["default"].is_null() {
			data["default"] = object!{};
		}

		Ok(Cache {
			display, data, path,
			profile: "default".into(),
		})
	}

	/// Save the cache to disk.
	pub fn save(&mut self) -> error::Result<()> {
		let mut file = File::create(&self.path)?;
		self.data.write_pretty(&mut file, 2)?;

		Ok(())
	}

	/// Change cache profile.
	pub fn profile<T: Into<String>>(&mut self, name: T) {
		self.profile = name.into();

		if self.data[&self.profile].is_null() {
			self.data[&self.profile] = object!{};
		}
	}

	/// Set the brightness value for the given mode.
	pub fn set(&mut self, mode: Mode, value: f32) -> error::Result<()> {
		match mode {
			Mode::Manual => {
				self.data[&self.profile]["manual"] = value.into();
			}

			// Just store the ID.
			Mode::Desktop(id) => {
				if self.data[&self.profile]["desktop"].is_null() {
					self.data[&self.profile]["desktop"] = object!{};
				}

				self.data[&self.profile]["desktop"][id.to_string()] = value.into();
			}

			// Store both the WM_CLASS instance and class name.
			Mode::Window(active) => {
				if let Some(id) = active {
					if self.data[&self.profile]["window"].is_null() {
						self.data[&self.profile]["window"] = object!{};
					}

					let name = xcbu::icccm::get_wm_class(&self.display, id).get_reply()?;

					self.data[&self.profile]["window"][name.instance()] = value.into();
					self.data[&self.profile]["window"][name.class()]    = value.into();
				}
			}

			// Store the luminance and brightness pairs in a sorted array.
			Mode::Luminance(luma) => {
				if self.data[&self.profile]["luminance"].is_null() {
					self.data[&self.profile]["luminance"] = array!{};
				}

				if let JsonValue::Array(ref mut array) = self.data[&self.profile]["luminance"] {
					// The luminance value is rounded and limited to an `u8` so the
					// actual settable luminance ranges are between 0 and 100 and don't
					// suffer bloating caused by precision errors.
					let luma = (luma * 50.0).round() as u8;

					// Just use binary search and insert/replace as told.
					match array.binary_search_by_key(&luma, |v| v[0].as_u8().unwrap()) {
						Ok(index) =>
							array[index] = array![luma, value],

						Err(index) =>
							array.insert(index, array![luma, value])
					}
				}
			}

			// Store the half hours since midnight in a sorted array.
			Mode::Time(time) => {
				if self.data[&self.profile]["time"].is_null() {
					self.data[&self.profile]["time"] = array!{};
				}

				if let JsonValue::Array(ref mut array) = self.data[&self.profile]["time"] {
					let halves = time.num_seconds_from_midnight() / (30 * 60);

					match array.binary_search_by_key(&halves, |v| v[0].as_u32().unwrap()) {
						Ok(index) =>
							array[index] = array![halves, value],

						Err(index) =>
							array.insert(index, array![halves, value]),
					}
				}
			}
		}

		Ok(())
	}

	/// Get the brightness value for the given mode.
	pub fn get(&mut self, mode: Mode) -> error::Result<Option<f32>> {
		match mode {
			Mode::Manual => {
				if let Some(value) = self.data[&self.profile]["manual"].as_f32() {
					return Ok(Some(value));
				}
			}

			// Desktop just checks the desktop ID.
			Mode::Desktop(id) => {
				if let Some(value) = self.data[&self.profile]["desktop"][id.to_string()].as_f32() {
					return Ok(Some(value))
				}
			}

			// Window checking first checks if the WM_CLASS instance name matches,
			// otherwise it uses the WM_CLASS class name.
			//
			// This allows specialization for a differently named window belonging to
			// the same class. (i.e. terminals using the same program but having
			// different settings)
			Mode::Window(active) => {
				if let Some(id) = active {
					let name = xcbu::icccm::get_wm_class(&self.display, id).get_reply()?;

					if let Some(value) = self.data[&self.profile]["window"][name.instance()].as_f32() {
						return Ok(Some(value));
					}

					if let Some(value) = self.data[&self.profile]["window"][name.class()].as_f32() {
						return Ok(Some(value));
					}
				}
			}

			// Fetching the brightness corresponding to the luminance is a little
			// convulted, but alas.
			Mode::Luminance(luma) => {
				if let JsonValue::Array(ref slice) = self.data[&self.profile]["luminance"] {
					// If the array is empty we can't do nuffin.
					if slice.is_empty() {
						return Ok(None);
					}

					// The luminance value is rounded and limited to an `u8` so the
					// actual settable luminance ranges are between 0 and 100 and don't
					// suffer bloating caused by precision errors.
					let luma = (luma * 50.0).round() as u8;

					// Since the luminance values are sorted we can do a binary search to
					// fetch the surrounding values.
					let index = match slice.binary_search_by_key(&luma, |v| v[0].as_u8().unwrap()) {
						Ok(index) | Err(index) => index
					};

					let before = slice.get(index.overflowing_sub(1).0).map(|v| (v[0].as_u8().unwrap(), v[1].as_f32().unwrap()));
					let after  = slice.get(index).map(|v| (v[0].as_u8().unwrap(), v[1].as_f32().unwrap()));

					match (before, after) {
						// This is not possible, it would mean the array was empty.
						(None, None) =>
							unreachable!(),

						// Just return the one value.
						(Some((_, value)), None) | (None, Some((_, value))) => {
							return Ok(Some(value));
						}

						// Use linear interpolation to get the proper brightness.
						(Some((g1, d1)), Some((g2, d2))) => {
							let g  = f32::from(luma);
							let g1 = f32::from(g1);
							let g2 = f32::from(g2);

							return Ok(Some(d1 + ((g - g1) / (g2 - g1)) * (d2 - d1)));
						}
					}
				}
			}

			// Fetching the time is also convulted.
			Mode::Time(time) => {
				if let JsonValue::Array(ref slice) = self.data[&self.profile]["time"] {
					if slice.is_empty() {
						return Ok(None);
					}

					let halves = time.num_seconds_from_midnight() / (30 * 60);

					// Since the halves are sorted we can do a binary search to fetch the
					// surrounding values.
					let index = match slice.binary_search_by_key(&halves, |v| v[0].as_u32().unwrap()) {
						Ok(index) | Err(index) => index
					};

					let before = slice.get(index.overflowing_sub(1).0).map(|v| (v[0].as_u32().unwrap(), v[1].as_f32().unwrap()));
					let after  = slice.get(index).map(|v| (v[0].as_u32().unwrap(), v[1].as_f32().unwrap()));

					match (before, after) {
						// This is not possible, it would mean the array was empty.
						(None, None) =>
							unreachable!(),

						// Just return the one value.
						(Some((_, value)), None) | (None, Some((_, value))) => {
							return Ok(Some(value));
						}

						// Use linear interpolation to get the proper brightness.
						(Some((g1, d1)), Some((g2, d2))) => {
							let g  = halves as f32;
							let g1 = g1 as f32;
							let g2 = g2 as f32;

							return Ok(Some(d1 + ((g - g1) / (g2 - g1)) * (d2 - d1)));
						}
					}
				}
			}
		}

		Ok(None)
	}
}

impl Drop for Cache {
	fn drop(&mut self) {
		self.save().unwrap();
	}
}
