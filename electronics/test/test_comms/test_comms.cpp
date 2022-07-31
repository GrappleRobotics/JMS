#include <Arduino.h>
#include <unity.h>

#include "DataPacket.h"
#include "Comms.h"

using namespace Comms;

void test_comms_start() {
  Comm::init((int)Comms::Message::Common::Device::Type::kMaster);
  Comm::start();
}

void test_comms_setBaudRate() {
  Comm::setBaudRate(115200, 500E3);
}

void test_comms_getDataFromAlliance() {
  Comms::Message::Nodes::ScoringTable s;
  s = Comms::Comm::getData(s);
}

void test_comms_sendDataToAlliance() {
  Comms::Message::Nodes::Alliance a;
  a.device.setType(Comms::Message::Common::Device::Type::kBlueDS);
  TEST_ASSERT_EQUAL(0, Comms::Comm::sendData(a));
}

void test_comms_getDataFromScoringTable() {
  Comms::Message::Nodes::Alliance a;
  a = Comms::Comm::getData(a);
}

void test_comms_sendDataToScoringTable() {
  Comms::Message::Nodes::ScoringTable s;
  s.device.setType(Comms::Message::Common::Device::Type::kMaster);
  TEST_ASSERT_EQUAL(0, Comms::Comm::sendData(s));
}

void test_comms_stop() {
  Comms::Comm::stop();
}

void setup() {
  // delay(2000);
  UNITY_BEGIN();
  RUN_TEST(test_comms_setBaudRate);
  RUN_TEST(test_comms_start);
  RUN_TEST(test_comms_getDataFromAlliance);
  RUN_TEST(test_comms_sendDataToAlliance);
  RUN_TEST(test_comms_getDataFromScoringTable);
  RUN_TEST(test_comms_sendDataToScoringTable);
  RUN_TEST(test_comms_stop);
}

void loop() {
  UNITY_END();
}