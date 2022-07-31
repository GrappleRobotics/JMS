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

void ScoringTable::onUpdate() {
  switch (_message2ScoringTable.lights.getMode()) {
    case Comms::Message::Common::Lights::Mode::kOff:
      _strip.set(CRGB(0,0,0));
      break;

    case Comms::Message::Common::Lights::Mode::kConstant:
      _strip.set(CRGB(_message2ScoringTable.lights.leds.r, _message2ScoringTable.lights.leds.g, _message2ScoringTable.lights.leds.b));
      break;

    case Comms::Message::Common::Lights::Mode::kPulse:
      _strip.setPulse(CRGB(_message2ScoringTable.lights.leds.r, _message2ScoringTable.lights.leds.g, _message2ScoringTable.lights.leds.b), _message2ScoringTable.lights.speed);
      break;

    case Comms::Message::Common::Lights::Mode::kChase:
      _strip.setWave(CRGB(_message2ScoringTable.lights.leds.r, _message2ScoringTable.lights.leds.g, _message2ScoringTable.lights.leds.b), 5, _message2ScoringTable.lights.speed);
      break;

    case Comms::Message::Common::Lights::Mode::kRainbow:
      _strip.setRainbow(_message2ScoringTable.lights.speed);
      break;
  }

  _message2BlueAlliance.lights = _message2ScoringTable.b_lights;
  _message2RedAlliance.lights = _message2ScoringTable.r_lights;

  Comm::sendData(_message2BlueAlliance);
  Comm::sendData(_message2RedAlliance);
}

void ScoringTable::init() {
  _strip.create<WS2812<2, GRB>>(120); // 120 led strips x 3
  _message2RedAlliance.device.setType(Message::Common::Device::Type::kRedDS);
  _message2BlueAlliance.device.setType(Message::Common::Device::Type::kBlueDS);
}

void ScoringTable::loop() {
  // _strip.setRainbow(2);
  _message2ScoringTable = Comm::getData(_message2ScoringTable, true);

  if (e_mst.isTriggered()) {
    _message2RedAlliance.field_estop = true;
    _message2BlueAlliance.field_estop = true;
    
    Comm::sendData(_message2BlueAlliance);
    Comm::sendData(_message2RedAlliance);
    _strip.setPulse(CRGB(255,0,0), 0);
  } else {
    onUpdate();
  }
}