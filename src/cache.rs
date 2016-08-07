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
use std::time::SystemTime;
use std::sync::Arc;

use xdg;
use json::{self, JsonValue};
use xcb;
use xcbu;

use {Display, error};

pub struct Cache {
	display: Arc<Display>,
	data:    JsonValue,
	path:    PathBuf,
	profile: String,
}

pub enum Mode {
	Manual,
	Desktop(i32),
	Window(Option<xcb::Window>),
	Luminance(f32),
	Time(SystemTime),
}

impl Cache {
	pub fn open<T: AsRef<Path>>(display: Arc<Display>, path: Option<T>) -> error::Result<Self> {
		let path = if let Some(path) = path {
			path.as_ref().into()
		}
		else {
			xdg::BaseDirectories::with_prefix("dux").unwrap()
				.place_config_file("cache.json").unwrap()
		};

		let mut data = if path.exists() {
			let mut file    = File::open(&path)?;
			let mut content = String::new();
			file.read_to_string(&mut content)?;

			json::parse(&content).unwrap_or(object!{})
		}
		else {
			object!{}
		};

		if data["default"].is_null() {
			data["default"] = object!{};
		}

		Ok(Cache {
			display: display,
			data:    data,
			path:    path,
			profile: "default".into(),
		})
	}

	pub fn save(&mut self) -> error::Result<()> {
		let mut file = File::create(&self.path)?;
		self.data.to_writer(&mut file);

		Ok(())
	}

	pub fn profile<T: Into<String>>(&mut self, name: T) {
		self.profile = name.into();

		if self.data[&self.profile].is_null() {
			self.data[&self.profile] = object!{};
		}
	}

	pub fn set(&mut self, mode: Mode, value: f32) -> error::Result<()> {
		match mode {
			Mode::Manual => (),

			Mode::Desktop(id) => {
				if self.data[&self.profile]["desktop"].is_null() {
					self.data[&self.profile]["desktop"] = object!{};
				}

				self.data[&self.profile]["desktop"][id.to_string()] = value.into();
			}

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

			Mode::Luminance(luma) => {
				if self.data[&self.profile]["luminance"].is_null() {
					self.data[&self.profile]["luminance"] = array!{};
				}

				if let JsonValue::Array(ref mut array) = self.data[&self.profile]["luminance"] {
					let luma = luma.round() as u8;

					match array.binary_search_by_key(&luma, |v| v[0].as_u8().unwrap()) {
						Ok(index) =>
							array[index] = array![luma, value],

						Err(index) =>
							array.insert(index, array![luma, value])
					}
				}
			}

			Mode::Time(time) => {
				// TODO: it
			}
		}

		Ok(())
	}

	pub fn get(&mut self, mode: Mode) -> error::Result<Option<f32>> {
		match mode {
			Mode::Manual => (),

			Mode::Desktop(id) => {
				if let Some(value) = self.data[&self.profile]["desktop"][id.to_string()].as_f32() {
					return Ok(Some(value))
				}
			}

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

			Mode::Luminance(luma) => {
				if let JsonValue::Array(ref slice) = self.data[&self.profile]["luminance"] {
					let luma = luma.round() as u8;

					if !slice.is_empty() {
						let index = match slice.binary_search_by_key(&luma, |v| v[0].as_u8().unwrap()) {
							Ok(index) | Err(index) => index
						};

						let before = slice.get(index.overflowing_sub(1).0).map(|v| (v[0].as_u8().unwrap(), v[1].as_f32().unwrap()));
						let after  = slice.get(index).map(|v| (v[0].as_u8().unwrap(), v[1].as_f32().unwrap()));

						return match (before, after) {
							(None, None) => {
								Ok(None)
							}

							(Some((_, value)), None) | (None, Some((_, value))) => {
								Ok(Some(value))
							}

							(Some((a1, a2)), Some((c1, c2))) => {
								let a1 = a1 as f32;
								let b1 = luma as f32;
								let c1 = c1 as f32;

								let p = {
									let x = b1 - a1;
									let y = c1 - a1;

									if a2 < c2 {
										(x * 100.0) / y
									}
									else {
										100.0 - ((x * 100.0) / y)
									}
								};

								Ok(Some({
									let x = f32::min(a2, c2);
									let y = f32::max(a2, c2);

									x + ((p * (y - x)) / 100.0)
								}))
							}
						};
					}
				}
			}

			Mode::Time(time) => {
				// TODO: it
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
