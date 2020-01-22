/**
 * File: main.rs
 * Date: 21.01.20
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

#[macro_use]
extern crate clap;

use std::error::Error as StdError;
use std::time::Duration;
use clap::{App, Arg};
use serial_unit_testing::serial::*;

mod error;
use error::*;

fn main() {
    let matches = App::new("dcc-quality-analyzer")
        .version(crate_version!())
        .version_short("v")
        .about(crate_description!())
        .arg(Arg::with_name("port")
            .help("Serial port OS specific name")
            .required(true)
            .takes_value(true))
        .get_matches();

    // open serial port
    let mut _serial = match connect_to_analyzer(matches.value_of("port").unwrap()) {
        Ok(serial) => serial,
        Err(err) => return eprintln!("Unable to connect to analyzer: {}", err.description())
    };
}

fn connect_to_analyzer(port_name: &str) -> Result<Serial> {
    let settings = settings::Settings {
        baud_rate: 115200,
        timeout: 3000,
        ..Default::default()
    };

    let mut serial = Serial::open_with_settings(port_name, &settings)?;

    // verify connection
    let identifier = read_str_until(&mut serial, "\n")?;
    if identifier.starts_with("DCC Quality Analyser V") == false {
        return Err(Error::UnknownDevice);
    }

    let pos = identifier.find("V");
    let version = if let Some(version_pos) = pos {
        let end_pos = identifier.find("\n").unwrap();
        &identifier[version_pos + 1..end_pos]
    } else {
        return Err(Error::UnknownDevice);
    };

    read_until_with_timeout(&mut serial, "Ready", Duration::from_millis(5000))?;

    println!("Analyzer hardware found, Version {}", version);

    Ok(serial)
}

fn read_str_until(serial: &mut Serial, desired: &str) -> Result<String> {
    let mut result = String::new();

    loop {
        match serial.read_str() {
            Ok(chunk) => result += &chunk,
            Err(ref err) if err.is_timeout() => return Err(Error::UnknownDevice),
            Err(err) => return Err(Error::Serial(err))
        }

        if result.contains(desired) {
            break;
        }
    }

    Ok(result)
}

fn read_until_with_timeout(serial: &mut Serial, desired: &str, timeout: Duration) -> Result<String> {
    let mut result = String::new();

    loop {
        match serial.read_str_with_timeout(timeout) {
            Ok(chunk) => result += &chunk,
            Err(ref err) if err.is_timeout() => return Err(Error::UnknownDevice),
            Err(err) => return Err(Error::Serial(err))
        }

        if result.contains(desired) {
            break;
        }
    }

    Ok(result)
}