#ifndef SCORING_TABLE_H
#define SCORING_TABLE_H


#include "NodeBase.h"
#include "InterruptButton.h"
#include "LEDStrips.h"

#include "DataPacket.h"


class ScoringTable : public NodeBase {
 public:
  ScoringTable(long serial_br = 115200, long can_br = 500E3);
  ~ScoringTable() {
    Comms::Comm::stop();
  }

  void init() override;
  void loop() override;

  void onUpdate();

 private:
  Comms::Message::Nodes::Alliance _message2RedAlliance;
  Comms::Message::Nodes::Alliance _message2BlueAlliance;
  Comms::Message::Nodes::ScoringTable _message2ScoringTable;

  LED::Strip _strip;

  static void onInterrupt_Estop(int station);

  static void onInterrupt_Emst();

  // Red E stops
  InterruptButton e_r1{4, []{onInterrupt_Estop(1);}};
  InterruptButton e_r2{5, []{onInterrupt_Estop(2);}};
  InterruptButton e_r3{6, []{onInterrupt_Estop(3);}};

  // // Blue E stops
  InterruptButton e_b1{7, []{onInterrupt_Estop(4);}};
  InterruptButton e_b2{8, []{onInterrupt_Estop(5);}};
  InterruptButton e_b3{9, []{onInterrupt_Estop(6);}};

  // // Filed E stop
  InterruptButton e_mst{A0, []{onInterrupt_Emst();}};
};

#endif