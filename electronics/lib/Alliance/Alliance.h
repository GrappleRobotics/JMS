#ifndef ALLIANCE_H
#define ALLIANCE_H

#include "NodeBase.h"
#include "LEDStrips.h"

#include "DataPacket.h"

class Alliance : public NodeBase {
 public:
  Alliance(Comms::Message::Common::Device::Type t, long serial_br = 115200, long can_br = 500E3);
  ~Alliance() {}

  void init() override;
  void loop() override;

 private:
  Comms::Message::Nodes::Alliance _message2Alliance;
  LED::Strip _strip;
};

#endif