#include <Arduino.h>
#include <unity.h>
#include "InterruptButton.h"

void onInterrupt() {}

void test_interrupt_button_pin() {
  InterruptButton b(A0, &onInterrupt);
  TEST_ASSERT_EQUAL(A0, b.getPin());
}

void test_interrupt_button_state() {
  InterruptButton b(A0, &onInterrupt);
  TEST_ASSERT_EQUAL(false, b.isTriggered());
}

void setup() {
  // delay(2000);
  UNITY_BEGIN();
  RUN_TEST(test_interrupt_button_pin);
  RUN_TEST(test_interrupt_button_state);
}

void loop() {
  UNITY_END();
}