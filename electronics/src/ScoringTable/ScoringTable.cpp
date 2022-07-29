#include <Arduino.h>
#include "ScoringTable.h"

void onInterrupt_Estop() {

}

void onInterrupt_Emst() {

}

ScoringTable::ScoringTable(long baudRate) : NodeBase(Comms::Message::Common::Device::Type::kMaster, (int)Comms::Message::Common::Device::Type::kMaster, baudRate) {
  
}

void ScoringTable::init() {
  _strip.create<WS2812<2, GRB>>(120); // 120 led strips x 3
  _message2RedAlliance.device.setType(Comms::Message::Common::Device::Type::kRedDS);
  _message2BlueAlliance.device.setType(Comms::Message::Common::Device::Type::kBlueDS);
}

void ScoringTable::loop() {
  _strip.setRainbow(2);
  if (e_mst.isTriggered()) {
    _message2RedAlliance.field_estop = true;
    // _message2BlueAlliance.field_estop = true;

    // Comms::Comm::sendDataTo(_message2BlueAlliance);
    Comms::Comm::sendDataTo(_message2RedAlliance);
    // _strip.setPulse(CRGB(255,0,0), 0);
  }
}