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

use std::thread;
use std::sync::mpsc::{Receiver, channel};
use std::ops::Deref;
use std::time::{Duration, Instant};

use error;

/// Timer handler.
pub struct Timer {
	receiver: Receiver<Event>,
}

#[derive(Debug)]
pub enum Event {
	/// The auto-save timer has fired.
	Save,
}

impl Timer {
	/// Spawn the `Timer` thread.
	pub fn spawn() -> error::Result<Self> {
		let (sender, receiver) = channel();

		thread::spawn(move || {
			let mut save = Instant::now();

			loop {
				thread::sleep(Duration::from_secs(1));

				if save.elapsed().as_secs() > 30 {
					save = Instant::now();
					sender.send(Event::Save).unwrap();
				}
			}
		});

		Ok(Timer {
			receiver: receiver,
		})
	}
}

impl Deref for Timer {
	type Target = Receiver<Event>;

	fn deref(&self) -> &Self::Target {
		&self.receiver
	}
}
