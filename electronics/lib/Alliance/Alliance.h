#ifndef ALLIANCE_H
#define ALLIANCE_H

#include "NodeBase.h"
#include "LEDStrips.h"

class Alliance : public NodeBase {
 public:
  Alliance(Comms::Message::Common::Device::Type t, long baudRate = k500Kbs);
  ~Alliance() {}

  void init() override;
  void loop() override;

 private:
  Comms::Message::Nodes::Alliance _message2Alliance;
  LED::Strip _strip;
};

#endif