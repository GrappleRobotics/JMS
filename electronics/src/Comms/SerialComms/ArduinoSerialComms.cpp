#ifdef ARDUINO

#include <Arduino.h>
#include "Comms/SerialComms/SerialComms.h"

void SerialComms::start(long baud) {
  Serial.begin(baud);
}

void SerialComms::write(uint8_t *buffer, size_t count) {
  Serial.write(buffer, count);
}

size_t SerialComms::available() {
  return Serial.available();
}

uint8_t SerialComms::read1() {
  return Serial.read();
}

uint8_t read(uint8_t *buffer, size_t count) {
  return Serial.readBytes(buffer, count);
}

#endif