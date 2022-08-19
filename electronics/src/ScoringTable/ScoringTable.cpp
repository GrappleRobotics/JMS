#include <Arduino.h>
#include "Comms/comms.h"
#include "Comms/SerialComms/SerialComms.h"
#include "ScoringTable/ScoringTable.h"

static Comms<SerialComms> serialComms;

volatile int triggered = 0;

void button_triggered(int e_stop) {
  EstopStates estops;
  switch (e_stop) {
    // Field E stop
    case 0:
      estops = {true, {false,false,false}, {false,false,false}};
      break;

    // Red E stops
    case 1:
      estops = {false, {true,false,false}, {false,false,false}};
      break;

    case 2:
      estops = {false, {false,true,false}, {false,false,false}};
      break;

    case 3:
      estops = {false, {false,false,true}, {false,false,false}};
      break;

    // Blue E stops
    case 4:
      estops = {false, {false,false,false}, {true,false,false}};
      break;

    case 5:
      estops = {false, {false,false,false}, {false,true,false}};
      break;

    case 6:
      estops = {false, {false,false,false}, {false,false,true}};
      break;
  }
}

ScoringTable::ScoringTable(unsigned long serial_br, unsigned long can_br) : NodeBase(serial_br, can_br) {}

void ScoringTable::init() {
  serialComms.start(_serial_br);
  e_mst = new InterruptButton(A2, []{button_triggered(0);}, true);

  e_r1 = new InterruptButton(4, []{button_triggered(1);}, true);
  e_r2 = new InterruptButton(5, []{button_triggered(2);}, true);
  e_r3 = new InterruptButton(6, []{button_triggered(3);}, true);

  e_b1 = new InterruptButton(7, []{button_triggered(4);}, true);
  e_b2 = new InterruptButton(8, []{button_triggered(5);}, true);
  e_b3 = new InterruptButton(9, []{button_triggered(6);}, true);
}

void ScoringTable::pollButtons() {
  const MessageEstops estop_message {
    EstopStates {
      // Field Estop
      e_mst->isTriggered(), 
      
      // Red Estops
      {
        e_r1->isTriggered(),
        e_r2->isTriggered(),
        e_r3->isTriggered()
      },

      // Blue Estops
      {
        e_b1->isTriggered(),
        e_b2->isTriggered(),
        e_b3->isTriggered()
      }
    }
  };

  AddressedMessage msg2jms {
    Role::ScoringTable,
    estop_message
  };

  serialComms.write(msg2jms);
}

void ScoringTable::pollLights() {
  auto msgFromJMS = serialComms.poll();

  // Checkers for jms serial
  if (!msgFromJMS.has_value()) return;
  if (msgFromJMS.get().role != Role::JMS) return;
  
  LightMode lights = msgFromJMS.get().msg.get<LightMode>();

  if (lights.is<LightModeConstant>()) {

  }
}

void ScoringTable::onUpdate() {
  pollButtons();
  // pollLights();
  delay(100);
}