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
    Reset
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

        let captures = match re.captures(raw) {
            Some(cap) => cap,
            None => return (DCCPacketType::Unknown, String::new())
        };

        data.push(DCCPacket::byte_string_to_u8(&captures["byte0"]));
        data.push(DCCPacket::byte_string_to_u8(&captures["byte1"]));
        data.push(DCCPacket::byte_string_to_u8(&captures["byte2"]));

        // TODO: Improve nesting
        if let Some(byte) = captures.name("byte3") {
            data.push(DCCPacket::byte_string_to_u8(byte.as_str()));

            if let Some(byte) = captures.name("byte4") {
                data.push(DCCPacket::byte_string_to_u8(byte.as_str()));

                if let Some(byte) = captures.name("byte5") {
                    data.push(DCCPacket::byte_string_to_u8(byte.as_str()));
                }
            }
        }

        // get packet type
        if data.len() == 3 && data[0] == 0xFF && data[1] == 0 && data[2] == 0xFF {
            return (DCCPacketType::Idle, "Idle packet".to_string());
        }

        if data.len() == 3 && data[0] == 0 && data[1] == 0 && data[2] == 0 {
            return (DCCPacketType::Reset, "Reset packet".to_string());
        }

        (DCCPacketType::Unknown, String::new())
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