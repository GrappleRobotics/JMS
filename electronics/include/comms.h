#pragma once

#include "messages.h"
#include "lib/optional.h"

#ifdef ARDUINO
#include <Arduino.h>
struct CommsSerial {
  static void start() {
    Serial.begin(115200);
  }

  static void write(uint8_t *buf, size_t count) {
    Serial.write(buf, count);
  }

  static size_t available() {
    return Serial.available();
  }

  static uint8_t read1() {
    return Serial.read();
  }

  static uint8_t read(uint8_t *buf, size_t count) {
    return Serial.readBytes(buf, count);
  }
};
#endif

template<class T>
class Comms {
 public:
  void start() {
    T::start();
    _to_read = 0;
    _n_read = 0;
  }

  void write(const AddressedMessage &msg) const {
    uint8_t data[64];
    auto it = &data[0] + 1;
    pack(msg, it);
    size_t count = it - (&data[0] + 1);

    if (count > 0 && count < 64) {
      data[0] = count;
      T::write(&data[0], count);
    }
  }

  optional<AddressedMessage> poll() const {
    size_t avail = T::available();
    if (avail > 0) {
      if (_to_read == 0) {
        // Read length
        _to_read = T::read1();
        _n_read = 0;
      } else {
        // Read message when available
        _n_read += T::read(&_buf[0] + _n_read, min(avail, _to_read - _n_read));
        if (_n_read >= _to_read) {
          // Full message read - unpack it
          AddressedMessage unpacked = unpack<AddressedMessage>(&_buf[0]);
          _n_read = 0;
          _to_read = 0;
          return unpacked;
        }
      }
    }
    return {};
  }

 private:
  size_t _to_read = 0;
  size_t _n_read = 0;
  uint8_t _buf[64];
};