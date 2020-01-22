/**
 * File: error.rs
 * Date: 22.01.20
 * Author: MarkAtk
 *
 * Copyright (c) 2020 MarkAtk
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is furnished to do
 * so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::convert::From;
use serial_unit_testing::error::Error as SerialError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    UnknownDevice,
    Serial(SerialError),
    Other
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::UnknownDevice => write!(f, "Unknown device"),
            Error::Serial(ref cause) => write!(f, "Serial Error: {}", cause.description()),
            Error::Other => write!(f, "Unknown error")
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnknownDevice => "Unknown analyzer device",
            Error::Serial(ref cause) => cause.description(),
            Error::Other => "Unknown error"
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Error::Serial(ref cause) => Some(cause),
            _ => None
        }
    }
}

impl From<SerialError> for Error {
    fn from(cause: SerialError) -> Error {
        Error::Serial(cause)
    }
}
