# DCC Quality Analyzer

## Description

The DCC Quality Analyzer is an Arduino based [DCC protocol](https://www.nmra.org/dcc-working-group) analyzer focusing on the quality of the signal. It provides features like bit level analyzing, error rate calculation, and more (full list below).

The analyzer is focused on developing dcc based systems but may also be used on existing installations.

The analyzer only works on digital side of the protocol. Errors on the analog side (like wrong e.g. voltage from a booster) will not be detected.

## Setup

A DCC booster signal **MUST NOT** be connected directly to the arduino, otherwise it will most probably destroy the arduino. Instead a compatible arduino dcc shield or custom circuit must be used to. Most shields output the DCC data onto *pin 2*, if another pin is used the sketch needs to be updated at `#define DCC_PIN`.

The host program is running on a pc connected to the arduino via USB. The program runs in the terminal and analyzes the incoming packages by type, calculates error rates and much more. The program is written in Rust and is cross-platform compatible. The host program can be downloaded from the [releases](https://github.com/markatk/dcc-quality-analyzer/releases) or compiled directly with Rust by running `cargo install` in the *quality analyzer* directory.

The host program is not strictly required since the arduino will output received data on the serial but it is strongly recommended, since all the interesting analyzation is done in the program.

## Usage

The host program is started by running `quality-analyzer <PORT>` in the terminal where <PORT> is the OS specific serial port of the connected arduino running the dcc-quality-analzer firmware.

## Features

- Bit level stream with false bit detection
- Error rate calculation
- Error trigger pin with adjustable threshold rate
- RailCom cutout detection
- Missing signal trigger pin with adjustable threshold rate
- Scope trigger pin for new packets
- Preamble counter

## License

MIT License

Copyright (c) 2019

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
