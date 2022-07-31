#include "SerialComms/SerialComms.h"

using namespace Comms;

long SerialComm::_baudRate = 115200;

void SerialComm::start() {
  Serial.begin(_baudRate);
}

void SerialComm::stop() {
  Serial.end();
}

void SerialComm::setBaudRate(long br) {
  _baudRate = br;
}