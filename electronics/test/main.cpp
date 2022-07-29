#include <Arduino.h>
#include <CAN.h>

#include <unity.h>

// Test cases
#include "test_button.h"
#include "test_comms.h"

void test_led_builtin_pin_number() {
  TEST_ASSERT_EQUAL(13, LED_BUILTIN);
}

void setup() {
  // delay(2000);

  Serial.begin(9600);
  UNITY_BEGIN();
  RUN_TEST(test_led_builtin_pin_number);

  test_button();
  test_comms();
}

void loop() {
  UNITY_END();
}