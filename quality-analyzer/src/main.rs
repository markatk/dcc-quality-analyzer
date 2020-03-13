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
use std::io::{self, stdout, Write, Stdout};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use clap::{App, Arg};
use serial_unit_testing::serial::*;
use tui::backend::CrosstermBackend;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode};
use crossterm::event::{self, KeyEvent, KeyCode, KeyModifiers, read};
use tui::Terminal;
use tui::widgets::{Widget, Block, Borders, Paragraph, Text};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Style, Modifier};

mod error;
use error::*;

struct Data {
    total_packets: u64,
    invalid_packets: u64,
    idle_packets: u64,
    reset_packets: u64,
    hardware: String,
    firmware: String,
    packets: Vec<DCCPacket>
}

impl Data {
    pub fn new(hardware: &str, firmware: &str) -> Data {
        Data {
            total_packets: 0,
            invalid_packets: 0,
            idle_packets: 0,
            reset_packets: 0,
            hardware: hardware.to_string(),
            firmware: firmware.to_string(),
            packets: vec![]
        }
    }

    pub fn add_packet(&mut self, raw: &str) {
        let packet = DCCPacket::new(raw);

        self.total_packets += 1;

        match packet.packet_type {
            DCCPacketType::Invalid => self.invalid_packets += 1,
            DCCPacketType::Idle => self.idle_packets += 1,
            DCCPacketType::Reset => self.reset_packets += 1,
            _ => ()
        };

        self.packets.push(packet);
    }
}

enum DCCPacketType {
    Unknown,
    Invalid,
    Idle,
    Reset
}

struct DCCPacket {
    raw: String,
    packet_type: DCCPacketType,
    description: String
}

impl DCCPacket {
    pub fn new(raw: &str) -> DCCPacket {
        DCCPacket {
            packet_type: DCCPacketType::Unknown,
            raw: raw.to_string(),
            description: "Unknown".to_string()
        }
    }

    pub fn to_text(&self) -> Vec<Text> {
        vec![
            Text::raw(&self.raw),
            Text::styled(format!(" ({})\n", self.description), Style::default().modifier(Modifier::ITALIC))
        ]
    }
}

enum Event {
    Input(KeyEvent)
}

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
    let (mut serial, firmware, hardware) = match connect_to_analyzer(matches.value_of("port").unwrap()) {
        Ok(serial) => serial,
        Err(err) => return eprintln!("Unable to connect to analyzer: {}", err.description())
    };

    // create window
    enable_raw_mode().expect("Unable to enter terminal raw mode");

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).expect("Unable to enter terminal alternate screen");

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Unable to create the terminal");
    terminal.hide_cursor().expect("Unable to hide cursor");

    let (tx, rx) = mpsc::channel();

    // start additional threads
    {
        let tx = tx.clone();
        thread::spawn(move || {
            loop {
                let event = match read() {
                    Ok(event) => event,
                    Err(err) => {
                        println!("Error in input thread: {}", err);

                        return;
                    }
                };

                match event {
                    event::Event::Key(key) => {
                        if let Err(_) = tx.send(Event::Input(key.clone())) {
                            return;
                        }
                    },
                    _ => {}
                }
            }
        });
    }

    // main loop
    let mut data = Data::new(&hardware, &firmware);
    data.add_packet("11111111111111 0 11111111 0 00000000 0 11111111 1");
    data.add_packet("11111111111111 0 00000000 0 00000000 0 00000000 1");

    loop {
        if let Err(err) = render(&mut terminal, &data) {
            eprintln!("Error rendering window: {}", err.description());

            break;
        }

        match rx.recv() {
            Ok(Event::Input(event)) => {
                match event {
                    KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::CONTROL } => {
                        if c == 'c' {
                            break;
                        }
                    },
                    KeyEvent { code: KeyCode::Char(c), modifiers: _ } => {
                        if c == 'q' {
                            break;
                        }
                    },
                    KeyEvent { code: KeyCode::Esc, modifiers: _ } => break,
                    _ => ()
                }
            },
            _ => ()
        };
    }

    // clean up terminal
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}

fn get_packet_percentage(count: u64, total: u64, default: f32) -> f32 {
    if total > 0 {
        count as f32 / total as f32 * 100.0
    } else {
        default
    }
}

fn render(terminal: &mut Terminal<CrosstermBackend<Stdout>>, data: &Data) -> std::result::Result<(), io::Error> {
    let total_packets = data.total_packets;
    let valid_packets = data.total_packets - data.invalid_packets;
    let invalid_packets = data.invalid_packets;
    let idle_packets = data.idle_packets;
    let reset_packets = data.reset_packets;

    let hardware = data.hardware.as_str();
    let firmware = data.firmware.as_str();

    let packets_text: Vec<Text> = data.packets
        .iter()
        .map(|packet| packet.to_text())
        .flatten()
        .collect();

    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(0),
                Constraint::Length(4)
            ].as_ref())
            .split(f.size());

        let statistics_text = vec![
            Text::raw(format!("  Total packets: {}\nValid packets: {}", total_packets, valid_packets)),
            Text::styled(format!(" ({}%)\n", get_packet_percentage(valid_packets, total_packets, 100.0)), Style::default().modifier(Modifier::ITALIC)),
            Text::raw(format!("Invalid packets: {} ", invalid_packets)),
            Text::styled(format!("({}%)\n", get_packet_percentage(invalid_packets, total_packets, 0.0)), Style::default().modifier(Modifier::ITALIC)),
            Text::raw(format!("   Idle packets: {} ", idle_packets)),
            Text::styled(format!("({}%)\n", get_packet_percentage(idle_packets, total_packets, 0.0)), Style::default().modifier(Modifier::ITALIC)),
            Text::raw(format!(" Reset packets: {} ", reset_packets)),
            Text::styled(format!("({}%)\n", get_packet_percentage(reset_packets, total_packets, 0.0)), Style::default().modifier(Modifier::ITALIC))
        ];

        let hardware_text = vec![
            Text::raw(format!("Hardware: {}\nFirmware: {}", hardware, firmware)),
        ];

        Paragraph::new(statistics_text.iter())
            .block(
                Block::default()
                    .title("Statistics")
                    .title_style(Style::default().modifier(Modifier::BOLD))
                    .borders(Borders::ALL))
            .wrap(true)
            .render(&mut f, chunks[0]);

        Paragraph::new(packets_text.iter())
            .block(
                Block::default()
                    .title("Packets")
                    .title_style(Style::default().modifier(Modifier::BOLD))
                    .borders(Borders::ALL))
            .wrap(false)
            .render(&mut f, chunks[1]);

        Paragraph::new(hardware_text.iter())
            .block(
                Block::default()
                    .title("Hardware")
                    .title_style(Style::default().modifier(Modifier::BOLD))
                    .borders(Borders::ALL))
            .wrap(false)
            .render(&mut f, chunks[2]);
    })
}

fn connect_to_analyzer(port_name: &str) -> Result<(Serial, String, String)> {
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
    let firmware_version = if let Some(version_pos) = pos {
        let end_pos = identifier.find("\n").unwrap();
        &identifier[version_pos + 1..end_pos]
    } else {
        return Err(Error::UnknownDevice);
    };

    read_until_with_timeout(&mut serial, "Ready", Duration::from_millis(5000))?;

    Ok((serial, firmware_version.to_string(), "Arduino Uno V3".to_string()))
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