#include <Arduino.h>
#include "Comms/comms.h"
#include "Comms/SerialComms/SerialComms.h"
#include "ScoringTable/ScoringTable.h"

static Comms<SerialComms> serialComms;

void button_triggered() {
  // Check all buttons and send
  // serialComms.
}

ScoringTable::ScoringTable(long serial_br, long can_br) : NodeBase(serial_br, can_br) {
  serialComms.start(_serial_br);
}

void ScoringTable::init() {
  e_mst = new InterruptButton(A2, []{button_triggered();}, true);

  e_r1 = new InterruptButton(4, []{button_triggered();}, true);
  e_r2 = new InterruptButton(5, []{button_triggered();}, true);
  e_r3 = new InterruptButton(6, []{button_triggered();}, true);

  e_b1 = new InterruptButton(7, []{button_triggered();}, true);
  e_b2 = new InterruptButton(8, []{button_triggered();}, true);
  e_b3 = new InterruptButton(9, []{button_triggered();}, true);
}

void ScoringTable::onUpdate() {

}