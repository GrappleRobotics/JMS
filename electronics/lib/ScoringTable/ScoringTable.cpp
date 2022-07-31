#include <Arduino.h>
#include "ScoringTable.h"

using namespace Comms;

ScoringTable::ScoringTable(long serial_br, long can_br) : NodeBase((unsigned int)Message::Common::Device::Type::kMaster) {
  Comm::setBaudRate(serial_br, can_br);
  Comm::start();
}

void ScoringTable::onInterrupt_Estop(int station) {
  Message::Nodes::JMS message;
  switch (station) {
    case 1:
      message.b1_estop = true;
      break;

    case 2:
      message.b2_estop = true;
      break;

    case 3:
      message.b3_estop = true;
      break;

    case 4:
      message.r1_estop = true;
      break;

    case 5:
      message.r2_estop = true;
      break;

    case 6:
      message.r3_estop = true;
      break;

    default:
      message.estop = true;
  }

  Comm::sendData(message);
};

void ScoringTable::onInterrupt_Emst() {
  Message::Nodes::JMS message;
  message.estop = true;
  Comm::sendData(message);
};

void ScoringTable::init() {
  _strip.create<WS2812<2, GRB>>(120); // 120 led strips x 3
  _message2RedAlliance.device.setType(Message::Common::Device::Type::kRedDS);
  _message2BlueAlliance.device.setType(Message::Common::Device::Type::kBlueDS);
}

void ScoringTable::loop() {
  _strip.setRainbow(2);
  if (e_mst.isTriggered()) {
    _message2RedAlliance.field_estop = true;
    _message2BlueAlliance.field_estop = true;
    
    Comm::sendData(_message2BlueAlliance);
    Comm::sendData(_message2RedAlliance);
    _strip.setPulse(CRGB(255,0,0), 0);
  }
}