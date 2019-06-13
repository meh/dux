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
use std::ops::Deref;
use std::time::{Duration, Instant};

use crate::error;
use channel::{self, Receiver, Sender, SendError};

/// Timer handler.
pub struct Timer {
	receiver: Receiver<Event>,
	refresh:  Sender<u64>,
}

#[derive(Debug)]
pub enum Event {
	/// Sent every `refresh` milliseconds.
	Refresh,

	/// Hurts my kokoro.
	Heartbeat,

	/// The auto-save timer has fired.
	Save,
}

#[derive(Copy, Clone, Debug)]
pub struct Settings {
	pub save:      u64,
	pub heartbeat: u64,
}

impl Timer {
	/// Spawn the `Timer` thread.
	pub fn spawn(settings: Settings) -> error::Result<Self> {
		let (sender, receiver)   = channel::unbounded();
		let (refresh, refresher) = channel::unbounded();

		// Spawn the refresh timer.
		{
			let sender = sender.clone();

			thread::spawn(move || {
				while let Ok(value) = refresher.recv() {
					thread::sleep(Duration::from_millis(value));
					sender.send(Event::Refresh).unwrap();
				}
			});
		}

		// Spawn the constant timers.
		thread::spawn(move || {
			let mut save = Instant::now();
			let mut beat = Instant::now();

			loop {
				thread::sleep(Duration::from_secs(1));

				if save.elapsed().as_secs() >= settings.save {
					save = Instant::now();
					sender.send(Event::Save).unwrap();
				}

				if beat.elapsed().as_secs() >= settings.heartbeat {
					beat = Instant::now();
					sender.send(Event::Heartbeat).unwrap();
				}
			}
		});

		Ok(Timer {
			receiver: receiver,
			refresh:  refresh,
		})
	}

	/// Request a refresh after the given milliseconds.
	pub fn refresh(&self, value: u64) -> Result<(), SendError<u64>> {
		self.refresh.send(value)
	}
}

impl Deref for Timer {
	type Target = Receiver<Event>;

	fn deref(&self) -> &Self::Target {
		&self.receiver
	}
}
