/**
 * File: dcc.rs
 * Date: 13.03.20
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

use tui::widgets::Text;
use tui::style::{Style, Modifier};
use regex::Regex;

pub enum DCCPacketType {
    Unknown,
    Invalid,
    Idle,
    Reset,
    Speed
}

pub struct DCCPacket {
    raw: String,
    pub packet_type: DCCPacketType,
    description: String
}

impl DCCPacket {
    pub fn new(raw: &str) -> DCCPacket {
        let (packet_type, description) = DCCPacket::analyze(raw);

        DCCPacket {
            packet_type,
            raw: raw.to_string(),
            description
        }
    }

    pub fn to_text(&self) -> Vec<Text> {
        vec![
            Text::raw(&self.raw),
            Text::styled(format!(" ({})\n", self.description), Style::default().modifier(Modifier::ITALIC))
        ]
    }

    fn analyze(raw: &str) -> (DCCPacketType, String) {
        // find binary data, start searching at the end
        let re = Regex::new(r"1{10,}\s0\s(?P<byte0>[01]{8})\s0\s(?P<byte1>[01]{8})\s0\s(?P<byte2>[01]{8})(\s0\s(?P<byte3>[01]{8}))?(\s0\s(?P<byte4>[01]{8}))?(\s0\s(?P<byte5>[01]{8}))?\s1").unwrap();
        let mut data: Vec<u8> = Vec::new();
        let unknown_type = (DCCPacketType::Unknown, "Unknown packet".to_string());

        let captures = match re.captures(raw) {
            Some(cap) => cap,
            None => return unknown_type
        };

        for i in 0..6 {
            let group = format!("byte{}", i);

            if let Some(byte) = captures.name(&group) {
                data.push(DCCPacket::byte_string_to_u8(byte.as_str()));
            } else {
                break;
            }
        }

        // verify error byte
        let mut error_byte = 0;
        for i in 0..data.len() - 1 {
            error_byte ^= data[i];
        }

        if error_byte != data[data.len() - 1] {
            return (DCCPacketType::Invalid, "Invalid packet".to_string());
        }

        // get packet type and parsed data
        if data.len() == 3 && data[0] == 0xFF && data[1] == 0 {
            return (DCCPacketType::Idle, "Idle packet".to_string());
        }

        if data.len() == 3 && data[0] == 0 && data[1] == 0 {
            return (DCCPacketType::Reset, "Reset packet".to_string());
        }

        if data.len() == 3 && data[0] & 0x7F != 0 && data[1] & 0x7F != 0 {
            let address = data[0] & 0x7F;

            let speed = match data[1] & 0x0F {
                0 => "stop".to_string(),
                1 => "e-stop".to_string(),
                _ => {
                    let mut value = ((data[1] & 0x0F) as usize * 2 + (data[1] & 0x10) as usize / 4) as i32;
                    if data[1] & 0x20 != 0 {
                        value = -value;
                    }

                    value.to_string()
                }
            };

            return (DCCPacketType::Speed, format!("Basic speed packet, address={}, speed={}", address, speed));
        }

        return unknown_type;
    }

    fn byte_string_to_u8(str: &str) -> u8 {
        let mut value = 0;

        for (i, c) in str.chars().enumerate() {
            if c != '0' {
                value |= 1 << (7 - i) as u8;
            }
        }

        return value;
    }
}
