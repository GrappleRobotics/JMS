#include <Arduino.h>
#include <unity.h>
#include "Alliance.h"

void test_alliance_constructors() {
  Alliance a(Comms::Message::Common::Device::Type::kBlueDS);
  a.~Alliance();
  Alliance b(Comms::Message::Common::Device::Type::kBlueDS);
  b.~Alliance();
}

void test_blue_alliance_functions() {
  Alliance a(Comms::Message::Common::Device::Type::kBlueDS);
  a.init();
  
  for (int i = 0; i < 2; i++) {
    a.loop();
  }
}

void test_red_alliance_functions() {
  Alliance a(Comms::Message::Common::Device::Type::kRedDS);
  a.init();
  
  for (int i = 0; i < 2; i++) {
    a.loop();
  }
}

void setup() {
  // delay(2000);
  UNITY_BEGIN();
  // RUN_TEST(test_alliance_constructors);
  RUN_TEST(test_blue_alliance_functions);
  RUN_TEST(test_red_alliance_functions);
}

void loop() {
  UNITY_END();
}