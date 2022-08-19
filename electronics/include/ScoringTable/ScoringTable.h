#pragma once

#include <InterruptButton.h>
#include "NodeBase/NodeBase.h"

class ScoringTable : public NodeBase {
 public:
  ScoringTable(unsigned long serial_br, unsigned long can_br);

  void init() override;
  void onUpdate() override;

  void pollLights();
  void pollButtons();

 private:
  InterruptButton *e_r1;
  InterruptButton *e_r2;
  InterruptButton *e_r3;

  InterruptButton *e_b1;
  InterruptButton *e_b2;
  InterruptButton *e_b3;

  InterruptButton *e_mst;
};