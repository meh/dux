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

mod error;
pub use error::Error;

mod display;
pub use display::Display;

mod screen;
pub use screen::Screen;

mod backlight;
pub use backlight::Backlight;

mod timer;
pub use timer::Timer;

mod interface;
pub use interface::Interface;

mod observer;
pub use observer::Observer;

mod cache;
pub use cache::Cache;

use std::sync::Arc;
use env_logger;
use clap::{ArgMatches, Arg, App, SubCommand};
use channel::select;

fn main() {
	env_logger::init();

	let     display   = Arc::new(Display::open().expect("no display found"));
	let mut backlight = backlight::open(display.clone()).expect("no backlight support");

	let mut app = App::new("dux")
		.version(env!("CARGO_PKG_VERSION"))
		.author("meh. <meh@schizofreni.co>")
		.subcommand(SubCommand::with_name("get")
			.about("Get the brightness percentage."))
		.subcommand(SubCommand::with_name("set")
			.about("Set the brightness percentage.")
			.arg(Arg::with_name("PERCENTAGE")
				.required(true)
				.index(1)
				.help("The new brightness value."))
			.arg(Arg::with_name("time")
				.short("t")
				.long("time")
				.takes_value(true)
				.help("Fade time in milliseconds (default is 200)."))
			.arg(Arg::with_name("steps")
				.short("s")
				.long("steps")
				.takes_value(true)
				.help("Number of steps in fade (default is 20).")))
		.subcommand(SubCommand::with_name("inc")
			.about("Increase the brightness percentage.")
			.arg(Arg::with_name("PERCENTAGE")
				.required(true)
				.index(1)
				.help("The new brightness value."))
			.arg(Arg::with_name("time")
				.short("t")
				.long("time")
				.takes_value(true)
				.help("Fade time in milliseconds (default is 0)."))
			.arg(Arg::with_name("steps")
				.short("s")
				.long("steps")
				.takes_value(true)
				.help("Number of steps in fade (default is 0).")))
		.subcommand(SubCommand::with_name("dec")
			.about("Decrease the brightness percentage.")
			.arg(Arg::with_name("PERCENTAGE")
				.required(true)
				.index(1)
				.help("The new brightness value."))
			.arg(Arg::with_name("time")
				.short("t")
				.long("time")
				.takes_value(true)
				.help("Fade time in milliseconds (default is 0)."))
			.arg(Arg::with_name("steps")
				.short("s")
				.long("steps")
				.takes_value(true)
				.help("Number of steps in fade (default is 0).")))
		.subcommand(SubCommand::with_name("adaptive")
			.about("Start adaptive brightness.")
			.arg(Arg::with_name("refresh")
				.short("R")
				.long("refresh")
				.takes_value(true)
				.help("Distance in milliseconds within which damages are collapsed (default is 500)."))
			.arg(Arg::with_name("threshold")
				.short("T")
				.long("threshold")
				.takes_value(true)
				.help("Minimum total number of pixels for damages to be collapsed (default is `160000` around `400x400`)."))
			.arg(Arg::with_name("time")
				.short("t")
				.long("time")
				.takes_value(true)
				.help("Time to sleep between each step (default is 5)."))
			.arg(Arg::with_name("step")
				.short("s")
				.long("step")
				.takes_value(true)
				.help("Step to increase the brightness by (default is 1.0)."))
			.arg(Arg::with_name("cache")
				.short("c")
				.long("cache")
				.takes_value(true)
				.help("The path to the cache file."))
			.arg(Arg::with_name("profile")
				.short("p")
				.long("profile")
				.takes_value(true)
				.help("The profile name (default is `default`)."))
			.arg(Arg::with_name("mode")
				.short("m")
				.long("mode")
				.takes_value(true)
				.help("One of either `desktop`, `window`, `luminance`, `time` or `manual.")))
		.subcommand(SubCommand::with_name("mode")
			.about("Change the adaption mode.")
			.arg(Arg::with_name("MODE")
				.required(true)
				.index(1)
				.help("One of either `desktop`, `window`, `luminance` or `time`.")))
		.subcommand(SubCommand::with_name("profile")
			.about("Change the adaption profile.")
			.arg(Arg::with_name("PROFILE")
				.required(true)
				.index(1)
				.help("The profile name.")))
		.subcommand(SubCommand::with_name("sync")
			.about("Synchronize any backlight changes with the adaptive daemon."))
		.subcommand(SubCommand::with_name("save")
			.about("Force flush the cache to disk."))
		.subcommand(SubCommand::with_name("stop")
			.about("Stop adaptive brightness mode."));

	let matches = app.clone().get_matches();
	match matches.subcommand() {
		("get", Some(submatches)) =>
			get(submatches, backlight),

		("set", Some(submatches)) =>
			set(submatches, backlight),

		("inc", Some(submatches)) =>
			inc(submatches, backlight),

		("dec", Some(submatches)) =>
			dec(submatches, backlight),

		("adaptive", Some(submatches)) =>
			adaptive(submatches, display, backlight),

		("mode", Some(submatches)) =>
			Interface::mode(submatches.value_of("MODE").unwrap()).unwrap(),

		("profile", Some(submatches)) =>
			Interface::profile(submatches.value_of("PROFILE").unwrap()).unwrap(),

		("sync", Some(_)) =>
			Interface::brightness(backlight.get().unwrap()).unwrap(),

		("save", Some(_)) =>
			Interface::save().unwrap(),

		("stop", Some(_)) =>
			Interface::stop().unwrap(),

		_ =>
			app.print_help().unwrap()
	}
}

pub fn get(_matches: &ArgMatches, mut backlight: Box<Backlight>) {
	println!("{:.2}", backlight.get().unwrap());
}

pub fn set(matches: &ArgMatches, mut backlight: Box<Backlight>) {
	let value = matches.value_of("PERCENTAGE").unwrap().parse().unwrap();
	let _     = Interface::brightness(value);

	backlight::fade::by_time(&mut backlight, value,
		matches.value_of("time").unwrap_or("200").parse().unwrap(),
		matches.value_of("steps").unwrap_or("20").parse().unwrap()).unwrap();
}

pub fn inc(matches: &ArgMatches, mut backlight: Box<Backlight>) {
	let value = backlight.get().unwrap() + matches.value_of("PERCENTAGE").unwrap().parse::<f32>().unwrap();
	let _     = Interface::brightness(value);

	backlight::fade::by_time(&mut backlight, value,
		matches.value_of("time").unwrap_or("0").parse().unwrap(),
		matches.value_of("steps").unwrap_or("0").parse().unwrap()).unwrap();
}

pub fn dec(matches: &ArgMatches, mut backlight: Box<Backlight>) {
	let value = backlight.get().unwrap() - matches.value_of("PERCENTAGE").unwrap().parse::<f32>().unwrap();
	let _     = Interface::brightness(value);

	backlight::fade::by_time(&mut backlight, value,
		matches.value_of("time").unwrap_or("0").parse().unwrap(),
		matches.value_of("steps").unwrap_or("0").parse().unwrap()).unwrap();
}

pub fn adaptive(matches: &ArgMatches, display: Arc<Display>, mut backlight: Box<Backlight>) {
	use std::time::{Duration, Instant};

	let time      = matches.value_of("time").unwrap_or("5").parse().unwrap();
	let step      = matches.value_of("step").unwrap_or("1.0").parse().unwrap();
	let refresh   = matches.value_of("refresh").unwrap_or("500").parse().unwrap();
	let threshold = if refresh > 0 {
		matches.value_of("threshold").unwrap_or("160000").parse().unwrap()
	}
	else {
		u64::max_value()
	};

	let     interface = Interface::spawn().unwrap();
	let     observer  = Observer::spawn(display.clone()).unwrap();
	let     timer     = Timer::spawn(timer::Settings { save: 30, heartbeat: 300 }).unwrap();
	let mut cache     = Cache::open(display.clone(), matches.value_of("cache")).unwrap();
	let mut screen    = Screen::open(display.clone(), display.width(), display.height()).unwrap();

	if let Some(profile) = matches.value_of("profile") {
		cache.profile(profile);
	}

	let mut mode        = interface::Mode::parse(matches.value_of("mode").unwrap_or("luminance")).unwrap();
	let mut active      = None;
	let mut desktop     = 0;
	let mut changed     = Instant::now() - Duration::from_secs(42);
	let mut brightness  = 0.0;
	let mut rated       = false;
	let mut screensaver = false;

	macro_rules! mode {
		($value:expr) =>(
			match $value {
				interface::Mode::Manual =>
					cache::Mode::Manual,

				interface::Mode::Desktop =>
					cache::Mode::Desktop(desktop),

				interface::Mode::Window =>
					cache::Mode::Window(active),

				interface::Mode::Luminance =>
					cache::Mode::Luminance(screen.luminance()),

				interface::Mode::Time =>
					cache::Mode::Time(chrono::Local::now()),
			}
		);
	}

	macro_rules! fade {
		($value:expr) => (
			match $value {
				Some(v) if v != brightness => {
					brightness = v;
					backlight::fade::by_step(&mut backlight, v, step, time)
				}

				_ => {
					Ok(())
				}
			}
		)
	}

	// XXX: select! is hicky
	let t = &*timer;
	let i = &*interface;
	let o = &*observer;

	loop {
		select! {
			recv(timer) -> event => {
				match event.unwrap() {
					timer::Event::Refresh => {
						rated = false;

						if mode == interface::Mode::Luminance {
							screen.flush().unwrap();

							if changed.elapsed().as_secs() >= 1 {
								fade!(cache.get(cache::Mode::Luminance(screen.luminance())).unwrap()).unwrap()
							}
						}
					}

					timer::Event::Heartbeat => {
						if mode == interface::Mode::Time {
							fade!(cache.get(cache::Mode::Time(chrono::Local::now())).unwrap()).unwrap();
						}
					}

					timer::Event::Save => {
						cache.save().unwrap();
					}
				}
			},

			recv(interface) -> event => {
				match event.unwrap() {
					interface::Event::Mode(value) => {
						mode = value;
						fade!(cache.get(mode!(value)).unwrap()).unwrap();
					}

					interface::Event::Profile(name) => {
						cache.profile(name);
					}
					
					interface::Event::Save => {
						cache.save().unwrap();
					}

					interface::Event::Brightness(value) => {
						changed = Instant::now();
						cache.set(mode!(mode), value).unwrap();
					}

					interface::Event::Stop => {
						break;
					}

					interface::Event::ScreenSaver(active) => {
						screensaver = active;
					}
				}
			},

			recv(observer) -> event => {
				match event.unwrap() {
					observer::Event::Show(_) | observer::Event::Hide(_) | observer::Event::Change(_) => (),

					observer::Event::Desktop(id) => {
						desktop = id;

						if mode == interface::Mode::Desktop {
							fade!(cache.get(cache::Mode::Desktop(desktop)).unwrap()).unwrap();
						}
					}

					observer::Event::Active(value) => {
						active = value;

						if mode == interface::Mode::Window {
							fade!(cache.get(cache::Mode::Window(active)).unwrap()).unwrap();
						}
					}

					observer::Event::Damage(rect) => {
						if mode == interface::Mode::Luminance && !screensaver {
							let refreshed = screen.damage(rect, threshold).unwrap();

							if !refreshed {
								if !rated {
									timer.refresh(refresh).unwrap();
									rated = true;
								}
							}
							else if changed.elapsed().as_secs() >= 1 {
								fade!(cache.get(cache::Mode::Luminance(screen.luminance())).unwrap()).unwrap()
							}
						}
					}

					observer::Event::Resize(width, height) => {
						screen.resize(width, height).unwrap();
					}
				}
			}
		}
	}
}
