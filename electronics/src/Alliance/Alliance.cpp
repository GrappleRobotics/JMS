#include "Alliance.h"

Alliance::Alliance(Comms::Message::Common::Device::Type t, long baudRate) : NodeBase(t, (int)t, baudRate) {
  if (t == Comms::Message::Common::Device::Type::kBlueDS) {
    _message2Alliance.lights.setLights(Comms::Message::Common::Lights::Mode::kConstant, {255,0,0,255});
  } else {
    _message2Alliance.lights.setLights(Comms::Message::Common::Lights::Mode::kConstant, {255,255,0,0});
  }
}

void Alliance::init() {
  _strip.create<WS2812<2, GRB>>(120); // 120 led strips x 3
}

void Alliance::loop() {
  switch (_message2Alliance.lights.getMode()) {
    case Comms::Message::Common::Lights::Mode::kOff:
      _strip.set(CRGB(0,0,0));
      break;

    case Comms::Message::Common::Lights::Mode::kConstant:
      _strip.set(CRGB(_message2Alliance.lights.leds.r, _message2Alliance.lights.leds.g, _message2Alliance.lights.leds.b));
      break;

    case Comms::Message::Common::Lights::Mode::kPulse:
      _strip.setPulse(CRGB(_message2Alliance.lights.leds.r, _message2Alliance.lights.leds.g, _message2Alliance.lights.leds.b), _message2Alliance.lights.speed);
      break;

    case Comms::Message::Common::Lights::Mode::kChase:
      _strip.setWave(CRGB(_message2Alliance.lights.leds.r, _message2Alliance.lights.leds.g, _message2Alliance.lights.leds.b), 5, _message2Alliance.lights.speed);
      break;

    case Comms::Message::Common::Lights::Mode::kRainbow:
      _strip.setRainbow(_message2Alliance.lights.speed);
      break;
  }

  if (_message2Alliance.field_estop) {
    _strip.setPulse(CRGB(255,0,0), 0);
  }
}