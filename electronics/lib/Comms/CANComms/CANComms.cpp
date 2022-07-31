#include <CAN.h>
#include "CANComms/CANComms.h"

#include "Serializer/DataSerializer.h"

using namespace Comms;

long CANComm::_baudRate = 500E3;
unsigned int CANComm::_id = 0;

void CANComm::start() {
  CAN.begin(_baudRate);
}

void CANComm::stop() {
  CAN.end();
}

void CANComm::setBaudRate(long br) {
  _baudRate = br;
}

void CANComm::setID(unsigned int id) {
  _id = id;
}