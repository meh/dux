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

use std::fmt;
use std::error;
use std::io;

use xcb;
use dbus;
use clap;
use json;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	Message(String),
	Unsupported,

	X(X),
	DBus(DBus),
	Cli(clap::Error),
	Json(json::JsonError),
}

#[derive(Debug)]
pub enum X {
	MissingExtension,
	Request(u8, u8),
	Connection(xcb::ConnError),
}

#[derive(Debug)]
pub enum DBus {
	AlreadyRegistered,
	Internal(dbus::Error),
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error::Io(value)
	}
}

impl From<String> for Error {
	fn from(value: String) -> Self {
		Error::Message(value)
	}
}

impl From<()> for Error {
	fn from(_: ()) -> Self {
		Error::Message("Something happened :(".into())
	}
}

impl From<X> for Error {
	fn from(value: X) -> Error {
		Error::X(value)
	}
}

impl From<xcb::ConnError> for Error {
	fn from(value: xcb::ConnError) -> Error {
		Error::X(X::Connection(value))
	}
}

impl<T> From<xcb::Error<T>> for Error {
	fn from(value: xcb::Error<T>) -> Error {
		Error::X(X::Request(value.response_type(), value.error_code()))
	}
}

impl From<dbus::Error> for Error {
	fn from(value: dbus::Error) -> Self {
		Error::DBus(DBus::Internal(value))
	}
}

impl From<clap::Error> for Error {
	fn from(value: clap::Error) -> Self {
		Error::Cli(value)
	}
}

impl From<json::JsonError> for Error {
	fn from(value: json::JsonError) -> Self {
		Error::Json(value)
	}
}

impl From<DBus> for Error {
	fn from(value: DBus) -> Self {
		Error::DBus(value)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
		f.write_str(error::Error::description(self))
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		match *self {
			Error::Io(ref err) =>
				err.description(),

			Error::Message(ref msg) =>
				msg.as_ref(),

			Error::Unsupported =>
				"Missing backlight support.",

			Error::X(ref err) => match *err {
				X::Request(..) =>
					"An X request failed.",

				X::MissingExtension =>
					"A required X extension is missing.",

				X::Connection(..) =>
					"Connection to the X display failed.",
			},

			Error::DBus(ref err) => match *err {
				DBus::AlreadyRegistered =>
					"The name has already been registered.",

				DBus::Internal(ref err) =>
					err.description(),
			},

			Error::Cli(ref err) =>
				err.description(),

			Error::Json(ref err) =>
				err.description(),
		}
	}
}
