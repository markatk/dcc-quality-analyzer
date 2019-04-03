/*
 * File: dcc-quality-analyzer.ino
 * Date: 01.04.2019
 * Author: MarkAtk
 *
 * MIT License
 *
 * Copyright (c) 2019 MarkAtk
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

////////////////////////////////////////
// Configuration parameters
////////////////////////////////////////

#define DCC_PIN 2
#define ERR_INT_PIN 4

////////////////////////////////////////
// Global variables and definitions
////////////////////////////////////////

#define BIT_STATE_FIRST_HALF_ONE 0x01
#define BIT_STATE_FIRST_HALF_ZERO 0x03
#define BIT_STATE_FIRST_HALF_FAILURE 0x02
#define BIT_STATE_SECOND_HALF_ONE 0x10
#define BIT_STATE_SECOND_HALF_ZERO 0x30
#define BIT_STATE_SECOND_HALF_FAILURE 0x20

#define PACKET_STATE_UNKNOWN 0x00
#define PACKET_STATE_PREAMBLE 0x01
#define PACKET_STATE_DATA_SEPARATOR 0x02
#define PACKET_STATE_DATA 0x03
#define PACKET_STATE_END 0x04

#define BUFFER_SIZE 128
#define MIN_PREAMBLE_COUNT 10
#define VERSION "1.0"

#define IS_ZERO_BIT(x) _buffer[x] == (BIT_STATE_FIRST_HALF_ZERO | BIT_STATE_SECOND_HALF_ZERO)
#define IS_ONE_BIT(x) _buffer[x] == (BIT_STATE_FIRST_HALF_ONE | BIT_STATE_SECOND_HALF_ONE)

#define PRINT_TOGGLE_STATE(x, y) Serial.println(x ? "\n" y " ON" : "\n" y " OFF")

static volatile uint8_t _buffer[BUFFER_SIZE] = {0};
static volatile uint8_t _bufferRead = 0;
static volatile uint8_t _bufferWrite = 0;

static volatile unsigned long _lastMicros = 0;
static volatile uint8_t _bitState;

static uint8_t _bitCounter = 0;
static uint8_t _packetState = PACKET_STATE_UNKNOWN;
static uint8_t _packetPreambleCount = 0;
static uint8_t _packetDataBitCount = 0;
static uint8_t _packetErrors = 0;
static uint32_t _totalPackets = 0;
static uint32_t _errorPackets = 0;
static uint32_t _totalErrors = 0;

static bool _printBitStream = false;
static bool _smartBitSeparator = true;
static bool _smartLineBreak = true;
static bool _errorInterrupt = false;

static unsigned long _lastUpdatePrint = 0;

////////////////////////////////////////
// Functions
////////////////////////////////////////

void handleInput();
void printBuffer();

void externalInterruptHandler();

void setup() {
    Serial.begin(115200);

    pinMode(DCC_PIN, INPUT);
    pinMode(ERR_INT_PIN, 4);

    digitalWrite(ERR_INT_PIN, LOW);

    attachInterrupt(digitalPinToInterrupt(DCC_PIN), externalInterruptHandler, CHANGE);

    for (auto i = 0; i < BUFFER_SIZE; i++) {
        _buffer[i] = 0;
    }

    Serial.print("DCC Quality Analyser V");
    Serial.println(VERSION);

    Serial.println("Commands: b - toggle bit stream, l - legend, s - toggle smart bit separator, S - toggle smart line break");
}

void loop() {
    handleInput();
    printBuffer();
}

void handleInput() {
    if (Serial.available() == 0) {
        return;
    }

    auto ch = Serial.read();
    switch (ch) {
        case 'b':
            _printBitStream = !_printBitStream;

            PRINT_TOGGLE_STATE(_printBitStream, "Print bit stream");

            break;

        case 'l':
            Serial.println();
            Serial.println("Legend:");
            Serial.println("\t1: Valid bit, value = 1");
            Serial.println("\t0: Valid bit, value = 0");
            Serial.println("\te: Invalid bit, first half 1, second half 0");
            Serial.println("\tE: Invalid bit, first half 0, second half 1");
            Serial.println("\tf: Invalid bit, first half invalid, second half 1");
            Serial.println("\tF: Invalid bit, first half invalid, second half 0");
            Serial.println("\ts: Invalid bit, first half 1, second half invalid");
            Serial.println("\tS: Invalid bit, first half 0, second half invalid");
            Serial.println("\tb: Invalid bit, first half invalid, second half invalid");
            Serial.println("\tu: Unknown state");

            break;

        case 's':
            _smartBitSeparator = !_smartBitSeparator;

            PRINT_TOGGLE_STATE(_smartBitSeparator, "Smart bit separation");

            break;

        case 'S':
            _smartLineBreak = !_smartLineBreak;

            PRINT_TOGGLE_STATE(_smartLineBreak, "Smart line break");

            break;

        case 'e':
            _errorInterrupt = !_errorInterrupt;

            PRINT_TOGGLE_STATE(_errorInterrupt, "Error interrupt");

            break;

        case '\n':
        case '\r':
            break;

        default:
            Serial.println("\nUnknown command");

            break;
    }
}

void printBit(uint8_t oldPacketState) {
    if (_printBitStream == false) {
        return;
    }

    if (_smartBitSeparator) {
        if ((oldPacketState == PACKET_STATE_PREAMBLE && _packetState == PACKET_STATE_DATA_SEPARATOR) ||
            (oldPacketState == PACKET_STATE_DATA && (_packetState == PACKET_STATE_DATA_SEPARATOR || _packetState == PACKET_STATE_END))) {
            Serial.print(" ");
        }
    }

    switch (_buffer[_bufferRead]) {
        case BIT_STATE_FIRST_HALF_ONE | BIT_STATE_SECOND_HALF_ONE:
            Serial.print("1");
            break;

        case BIT_STATE_FIRST_HALF_ZERO | BIT_STATE_SECOND_HALF_ZERO:
            Serial.print("0");
            break;

        case BIT_STATE_FIRST_HALF_ONE | BIT_STATE_SECOND_HALF_ZERO:
            Serial.print("e");
            break;

        case BIT_STATE_FIRST_HALF_ZERO | BIT_STATE_SECOND_HALF_ONE:
            Serial.print("E");
            break;

        case BIT_STATE_FIRST_HALF_FAILURE | BIT_STATE_SECOND_HALF_ONE:
            Serial.print("f");
            break;

        case BIT_STATE_FIRST_HALF_FAILURE | BIT_STATE_SECOND_HALF_ZERO:
            Serial.print("F");
            break;

        case BIT_STATE_FIRST_HALF_ONE | BIT_STATE_SECOND_HALF_FAILURE:
            Serial.print("s");
            break;

        case BIT_STATE_FIRST_HALF_ZERO | BIT_STATE_SECOND_HALF_FAILURE:
            Serial.print("S");
            break;

        case BIT_STATE_FIRST_HALF_FAILURE | BIT_STATE_SECOND_HALF_FAILURE:
            Serial.print("b");
            break;

        default:
            Serial.print("u");
            Serial.print(_buffer[_bufferRead], HEX);
            break;
    }

    // handle spaces and line breaks
    if (_smartBitSeparator) {
        if (_packetState == PACKET_STATE_DATA_SEPARATOR || _packetState == PACKET_STATE_END) {
            Serial.print(" ");
        }
    } else {
        if (_bitCounter >= 8) {
            _bitCounter = 0;

            Serial.print(" ");
        }
    }

    if (_smartLineBreak && _packetState == PACKET_STATE_END) {
        Serial.println();
    }
}

void updatePacketState() {
    switch (_packetState) {
        case PACKET_STATE_UNKNOWN:
            if (IS_ONE_BIT(_bufferRead)) {
                _packetPreambleCount = 0;

                _packetState = PACKET_STATE_PREAMBLE;
            }

            break;

        case PACKET_STATE_PREAMBLE:
            if (IS_ZERO_BIT(_bufferRead) && _packetPreambleCount >= MIN_PREAMBLE_COUNT) {
                _packetState = PACKET_STATE_DATA_SEPARATOR;
            } else {
                _packetPreambleCount++;
            }

            break;

        case PACKET_STATE_DATA_SEPARATOR:
            _packetState = PACKET_STATE_DATA;

            break;

        case PACKET_STATE_DATA:
            if (_packetDataBitCount >= 7) {
                _packetDataBitCount = 0;

                if (IS_ZERO_BIT(_bufferRead)) {
                    _packetState = PACKET_STATE_DATA_SEPARATOR;
                } else if (IS_ONE_BIT(_bufferRead)) {
                    _packetState = PACKET_STATE_END;
                } else {
                    _packetState = PACKET_STATE_UNKNOWN;
                }
            } else {
                _packetDataBitCount++;
            }

            break;

        case PACKET_STATE_END:
            _packetPreambleCount = 0;

            if (IS_ONE_BIT(_bufferRead)) {
                _packetState = PACKET_STATE_PREAMBLE;
            } else {
                _packetState = PACKET_STATE_UNKNOWN;
            }

            break;
    }

    if (IS_ZERO_BIT(_bufferRead) == false && IS_ONE_BIT(_bufferRead) == false) {
        _packetErrors++;

        if (_errorInterrupt) {
            digitalWrite(ERR_INT_PIN, HIGH);
            delay(1);
            digitalWrite(ERR_INT_PIN, LOW);
        }
    }
}

void printBuffer() {
    uint8_t oldPacketState;

    while (_bufferRead != _bufferWrite) {
        // update packet state
        oldPacketState = _packetState;

        updatePacketState();

        if (_packetState == PACKET_STATE_END) {
            _totalPackets++;

            if (_packetErrors > 0) {
                _errorPackets++;
                _totalErrors += _packetErrors;
            }

            _packetErrors = 0;
        }

        // print bit representation
        _bitCounter++;

        printBit(oldPacketState);

        // advance buffer
        _buffer[_bufferRead] = 0;

        _bufferRead++;
        if (_bufferRead >= BUFFER_SIZE) {
            _bufferRead = 0;
        }
    }

    auto now = millis();
    if (_lastUpdatePrint == 0) {
        _lastUpdatePrint = now;

        return;
    }

    if (now - _lastUpdatePrint < 1000) {
        return;
    }

    Serial.print("Total packets: ");
    Serial.print(_totalPackets);
    Serial.print(", total errors: ");
    Serial.print(_totalErrors);
    Serial.print(", error packet rate: ");
    Serial.println((float) _errorPackets / (float) _totalPackets);

    _lastUpdatePrint = now;
}

void externalInterruptHandler() {
    auto _actualMicros = micros();

    // if first change, discard data since timing did not start yet
    if (_lastMicros == 0) {
        _lastMicros = _actualMicros;

        return;
    }

    auto _bitMicros = _actualMicros - _lastMicros;
    _lastMicros = _actualMicros;

    if (_bitMicros > 9999) {
        // reset packet
        _packetState = PACKET_STATE_UNKNOWN;
        _buffer[_bufferWrite] = 0;

        return;
    }

    if ((_buffer[_bufferWrite] & 0x0F) == 0) {
        // upper part
        if (_bitMicros < 52) {
            _buffer[_bufferWrite] |= BIT_STATE_FIRST_HALF_FAILURE;
        } else if (_bitMicros < 65) {
            _buffer[_bufferWrite] |= BIT_STATE_FIRST_HALF_ONE;
        } else {
            _buffer[_bufferWrite] |= BIT_STATE_FIRST_HALF_ZERO;
        }
    } else {
        // lower part
        if (_bitMicros < 52) {
            _buffer[_bufferWrite] |= BIT_STATE_SECOND_HALF_FAILURE;
        } else if (_bitMicros < 65) {
            _buffer[_bufferWrite] |= BIT_STATE_SECOND_HALF_ONE;
        } else {
            _buffer[_bufferWrite] |= BIT_STATE_SECOND_HALF_ZERO;
        }
    }

    // wait till both halves of the bit are set
    if ((_buffer[_bufferWrite] & 0x0F) == 0 || (_buffer[_bufferWrite] & 0xF0) == 0) {
        return;
    }

    // advance buffer
    _bufferWrite++;
    if (_bufferWrite >= BUFFER_SIZE) {
        _bufferWrite = 0;
    }
}

