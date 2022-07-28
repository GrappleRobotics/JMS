#ifndef ALLIANCE_H
#define ALLIANCE_H

#include "Comms/Comms.h"
#include "LEDStrips.h"

class Alliance {
 public:
  void init(Comms::Message::Common::Device::Type t, long baudRate = k500Kbs) {
    _type = t;
    _strip.create<WS2812<2, BRG>>(360); // 120 led strips x 3

    Comms::Comm::setBaudRate(baudRate);
    Comms::Comm::setNodeID(t);
    Comms::Comm::start();

    if (t == Comms::Message::Common::Device::Type::kBlueDs) {
      _message.lights.setLights(Comms::Message::Common::Lights::Mode::kConstant, {255,0,0,255});
    } else {
      _message.lights.setLights(Comms::Message::Common::Lights::Mode::kConstant, {255,255,0,0});
    }
  }

  ~Alliance() {
    Comms::Comm::stop();
  }

  void loop() {
    switch (_message.lights.getMode()) {
      case Comms::Message::Common::Lights::Mode::kOff:
        _strip.set(CRGB(0,0,0));
        break;

      case Comms::Message::Common::Lights::Mode::kConstant:
        _strip.set(CRGB(_message.lights.leds.r, _message.lights.leds.g, _message.lights.leds.b));
        break;

      case Comms::Message::Common::Lights::Mode::kPulse:
        _strip.setPulse(CRGB(_message.lights.leds.r, _message.lights.leds.g, _message.lights.leds.b), _message.lights.speed);
        break;

      case Comms::Message::Common::Lights::Mode::kChase:
        _strip.setWave(CRGB(_message.lights.leds.r, _message.lights.leds.g, _message.lights.leds.b), 5, _message.lights.speed);
        break;

      case Comms::Message::Common::Lights::Mode::kRainbow:
        _strip.setRainbow(_message.lights.speed);
        break;
    }

    if (_message.field_estop) {
      _strip.setPulse(CRGB(255,0,0), 0);
    }
  }

 private:
  Comms::Message::Nodes::Alliance _message;
  Comms::Message::Common::Device::Type _type;
  LED::Strip _strip;
};

#endif