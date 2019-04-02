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

#define DCC_PIN 2

#define BUFFER_SIZE 128
#define VERSION "1.0"

static volatile uint8_t _buffer[BUFFER_SIZE] = {0};
static volatile uint8_t _bufferRead = 0;
static volatile uint8_t _bufferWrite = 0;

static volatile unsigned long _lastMicros;
static volatile uint8_t _bitState;

static uint8_t _bitCounter = 0;

#define BIT_STATE_FIRST_HALF_ONE 0x01
#define BIT_STATE_FIRST_HALF_ZERO 0x03
#define BIT_STATE_FIRST_HALF_FAILURE 0x02
#define BIT_STATE_SECOND_HALF_ONE 0x10
#define BIT_STATE_SECOND_HALF_ZERO 0x30
#define BIT_STATE_SECOND_HALF_FAILURE 0x20

void externalInterruptHandler() {
    unsigned long _actualMicros = micros();
    unsigned long _bitMicros = _actualMicros - _lastMicros;

    _lastMicros = _actualMicros;

    uint8_t val = digitalRead(DCC_PIN);

    // first half (upper part)
    if (val == LOW) {
        _buffer[_bufferWrite] = 0;
      
        if (_bitMicros < 52) {
            _buffer[_bufferWrite] |= BIT_STATE_FIRST_HALF_FAILURE;
        } else if (_bitMicros < 65) {
            _buffer[_bufferWrite] |= BIT_STATE_FIRST_HALF_ONE;
        } else {
            _buffer[_bufferWrite] |= BIT_STATE_FIRST_HALF_ZERO;
        }

        return;
    }

    if (_buffer[_bufferWrite] == 0) {
        return;
    }
        
    // second half (lower part)
    if (_bitMicros < 52) {
        _buffer[_bufferWrite] |= BIT_STATE_SECOND_HALF_FAILURE;
    } else if (_bitMicros < 65) {
        _buffer[_bufferWrite] |= BIT_STATE_SECOND_HALF_ONE;
    } else {
        _buffer[_bufferWrite] |= BIT_STATE_SECOND_HALF_ZERO;
    }

    _bufferWrite++;
    if (_bufferWrite >= BUFFER_SIZE) {
        _bufferWrite = 0;
    }
}

void setup() {
    Serial.begin(115200);
    
    pinMode(DCC_PIN, INPUT);
    
    attachInterrupt(digitalPinToInterrupt(DCC_PIN), externalInterruptHandler, CHANGE);
    
    Serial.print("DCC Quality Analyser V");
    Serial.print(VERSION);

    Serial.println("Commands: b - line break, l - legend");
}

void loop() {
    if (Serial.available() > 0) {
        char ch = Serial.read();

        switch (ch) {
            case 'b':
                Serial.println();
                _bitCounter = 0;
                
                break;

            case 'l':
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

            case '\n':
            case '\r':
                break;

            default:
                Serial.println("\nUnknown command");
                break;
        }
    }
  
    while (_bufferRead != _bufferWrite) {
        uint8_t val = _buffer[_bufferRead];

        switch (val) {
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
                break;
        }

        _buffer[_bufferRead] = 0;

        _bitCounter++;
        if (_bitCounter >= 8) {
            _bitCounter = 0;
  
            Serial.print(" ");
        }

        _bufferRead++;
        if (_bufferRead >= BUFFER_SIZE) {
            _bufferRead = 0;
        }
    }
}
