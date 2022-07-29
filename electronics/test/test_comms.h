#include <Arduino.h>
#include <CAN.h>

#include <unity.h>

#include "Comms/Comms.h"

void test_comms_start() {
  Comms::Comm::start();
}

void test_comms_setBaudRate() {
  Comms::Comm::setBaudRate(k500Kbs);
}

void test_comms_setNodeID() {
  Comms::Comm::setNodeID(Comms::Message::Common::Device::Type::kMaster);
}

void test_comms_getDataFromAlliance() {
  Comms::Message::Nodes::Alliance a;
  a = Comms::Comm::getData(a);
}

void test_comms_sendDataToAlliance() {
  Comms::Message::Nodes::Alliance a;
  a.device.setType(Comms::Message::Common::Device::Type::kBlueDS);
  TEST_ASSERT_EQUAL(0, Comms::Comm::sendDataTo(a));
}

void test_comms_stop() {
  Comms::Comm::stop();
}

void test_comms() {
  RUN_TEST(test_comms_setBaudRate);
  RUN_TEST(test_comms_start);
  RUN_TEST(test_comms_setNodeID);
  RUN_TEST(test_comms_getDataFromAlliance);
  RUN_TEST(test_comms_sendDataToAlliance);
  RUN_TEST(test_comms_stop);
}