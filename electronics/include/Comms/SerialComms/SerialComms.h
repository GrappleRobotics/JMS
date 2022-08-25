#pragma once

#include <Arduino.h>
#include <stddef.h>
#include <stdint.h>

struct SerialComms {
  /**
   * @brief Start the serial comms
   * 
   * @param baud 
   */
  static void start(long baud) {
    Serial.begin(baud);
  }

  /**
   * @brief write 
   * 
   * @param buffer 
   * @param count 
   */
  static void write(uint8_t *buffer, size_t count) {
    Serial.write(buffer, count);
  }

  /**
   * @brief return how many bytes available
   * 
   * @return size_t 
   */
  static size_t available() {
    return Serial.available();
  }

  /**
   * @brief read one byte
   * 
   * @return uint8_t 
   */
  static uint8_t read1() {
    return Serial.read();
  }

  /**
   * @brief read into buffer
   * 
   * @param buffer 
   * @param count 
   * @return size of message in bytes (uint8_t)
   */
  static uint8_t read(uint8_t *buffer, size_t count) {
    return Serial.readBytes(buffer, count);
  }
};