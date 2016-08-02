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

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
use clap::{ArgMatches, Arg, App, SubCommand};

extern crate xdg;
extern crate toml;
extern crate dbus;

extern crate xcb;
extern crate xcb_util as xcbu;
extern crate byteorder;

use std::rc::Rc;

mod backlight;
use backlight::Backlight;

fn main() {
	env_logger::init().unwrap();

	let (connection, screen) = xcb::Connection::connect(None).map(|(c, s)| (Rc::new(c), s)).expect("no display found");
	let backlight            = backlight::open(connection.clone(), screen).expect("no backlight support");

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
				.help("Number of steps in fade (default is 0).")));

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

		_ =>
			app.print_help().unwrap()
	}
}

pub fn get(_matches: &ArgMatches, mut backlight: Box<Backlight>) {
	println!("{:.2}", backlight.get());
}

pub fn set(matches: &ArgMatches, mut backlight: Box<Backlight>) {
	fade(&mut backlight,
		matches.value_of("PERCENTAGE").unwrap().parse().unwrap(),
		matches.value_of("time").unwrap_or("200").parse().unwrap(),
		matches.value_of("steps").unwrap_or("20").parse().unwrap());
}

pub fn inc(matches: &ArgMatches, mut backlight: Box<Backlight>) {
	let current = backlight.get();

	fade(&mut backlight,
		current + matches.value_of("PERCENTAGE").unwrap().parse::<f32>().unwrap(),
		matches.value_of("time").unwrap_or("0").parse().unwrap(),
		matches.value_of("steps").unwrap_or("0").parse().unwrap());
}

pub fn dec(matches: &ArgMatches, mut backlight: Box<Backlight>) {
	let current = backlight.get();

	fade(&mut backlight,
		current - matches.value_of("PERCENTAGE").unwrap().parse::<f32>().unwrap(),
		matches.value_of("time").unwrap_or("0").parse().unwrap(),
		matches.value_of("steps").unwrap_or("0").parse().unwrap());
}

fn fade(backlight: &mut Box<Backlight>, value: f32, time: i32, steps: i32) {
	use std::thread;
	use std::time::Duration;

	let value = if value > 100.0 {
		100.0
	}
	else if value < 0.0 {
		0.0
	}
	else {
		value
	};

	if steps != 0 && time != 0 {
		let mut current = backlight.get();
		let     step    = (value - current) as i32 / steps;

		for _ in 0 .. steps {
			current += step as f32;
			backlight.set(current as f32);
			thread::sleep(Duration::from_millis((time / steps) as u64));
		}
	}

	backlight.set(value);
}
