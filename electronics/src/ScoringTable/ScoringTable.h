#ifndef SCORING_TABLE_H
#define SCORING_TABLE_H

#include "NodeBase/NodeBase.h"
#include "InterruptButton.h"
#include "LEDStrips.h"

void onInterrupt_Estop();
void onInterrupt_Emst();

class ScoringTable : public NodeBase {
 public:
  ScoringTable(long baudRate);

  void init() override;
  void loop() override;

 private:
  Comms::Message::Nodes::Alliance _message2RedAlliance;
  Comms::Message::Nodes::Alliance _message2BlueAlliance;
  Comms::Message::Nodes::ScoringTable _message2ScoringTable;

  LED::Strip _strip;

  // Red E stops
  InterruptButton e_r1{4, &onInterrupt_Estop};
  InterruptButton e_r2{5, &onInterrupt_Estop};
  InterruptButton e_r3{6, &onInterrupt_Estop};

  // Blue E stops
  InterruptButton e_b1{7, &onInterrupt_Estop};
  InterruptButton e_b2{8, &onInterrupt_Estop};
  InterruptButton e_b3{9, &onInterrupt_Estop};

  // Filed E stop
  InterruptButton e_mst{A0, &onInterrupt_Emst, true};
};

#endif