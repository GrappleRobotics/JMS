#pragma once
#include "config.h"
#if NODE_TYPE==3
#include "NodeBase/NodeBase.h"
#include "LEDStrips.h"

class ScoringTableSlave : public NodeBase {
 public:
  ScoringTableSlave();

  void init() override;
  void onUpdate() override;

  void updateLights();

 private:
  LED::Strip _strip;

  uint8_t _mode = 0;
  uint8_t _rgb[3] = {0,0,0};
  uint8_t _duration = 0;
};
#endif