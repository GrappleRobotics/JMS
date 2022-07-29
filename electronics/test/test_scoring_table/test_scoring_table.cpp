#include <Arduino.h>
#include <unity.h>
#include "ScoringTable.h"

void test_scoring_table_constructors() {
  ScoringTable s(k500Kbs);
  s.~ScoringTable();
}

void test_scoring_table_functions() {
  ScoringTable s(k500Kbs);
  s.~ScoringTable();
  s.init();
  
  for (int i = 0; i < 2; i++) {
    s.loop();
  }
}

void setup() {
  // delay(2000);
  UNITY_BEGIN();
  RUN_TEST(test_scoring_table_functions);
}

void loop() {
  UNITY_END();
}