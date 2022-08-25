#include "ScoringTable/ScoringTable.h"
#if NODE_TYPE==0

#include <Arduino.h>
#include <SPI.h>

#include "Comms/SerialComms/SerialComms.h"
#include "Comms/comms.h"

static Comms<SerialComms> serialComms;

ScoringTable::ScoringTable(unsigned long serial_br, unsigned long can_br) : NodeBase(serial_br, can_br) {}

void ScoringTable::init() {
  serialComms.start(_serial_br);
  SPI.begin();
  SPI.setClockDivider(SPI_CLOCK_DIV8);
  e_mst = new InterruptButton(A2, []{}, true);

  e_r1 = new InterruptButton(4, []{}, true);
  e_r2 = new InterruptButton(5, []{}, true);
  e_r3 = new InterruptButton(6, []{}, true);

  e_b1 = new InterruptButton(7, []{}, true);
  e_b2 = new InterruptButton(8, []{}, true);
  e_b3 = new InterruptButton(9, []{}, true);
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
  // if (!msgFromJMS.has_value()) return;
  // if (msgFromJMS.get().role != Role::JMS) return;

  LightMode lights = msgFromJMS.get().msg.get<LightMode>();

  // Set the bytes for the slave
  uint8_t slaveData[5] = {0,0,0,0,0}; // mode,r,g,b,dur
  if (lights.is<LightModeOff>()) {
    slaveData[0] = 0;
  } else if (lights.is<LightModeConstant>()) {
    slaveData[0] = 1;
    slaveData[1] = lights.get<LightModeConstant>().colour.red;
    slaveData[2] = lights.get<LightModeConstant>().colour.green;
    slaveData[3] = lights.get<LightModeConstant>().colour.blue;
    slaveData[4] = 0;
  } else if (lights.is<LightModePulse>()) {
    slaveData[0] = 2;
    slaveData[1] = lights.get<LightModePulse>().colour.red;
    slaveData[2] = lights.get<LightModePulse>().colour.green;
    slaveData[3] = lights.get<LightModePulse>().colour.blue;
    slaveData[4] = lights.get<LightModePulse>().duration;
  } else if (lights.is<LightModeChase>()) {
    slaveData[0] = 3;
    slaveData[1] = lights.get<LightModeChase>().colour.red;
    slaveData[2] = lights.get<LightModeChase>().colour.green;
    slaveData[3] = lights.get<LightModeChase>().colour.blue;
    slaveData[4] = lights.get<LightModeChase>().duration;
  } else if (lights.is<LightModeRainbow>()) {
    slaveData[0] = 4;
    slaveData[4] = lights.get<LightModeRainbow>().duration;
  } else {
    slaveData[0] = 0;
  }
  
  digitalWrite(SS, LOW);
  SPI.transfer(slaveData, 5);
  digitalWrite(SS, HIGH);

  // Test
  slaveData[0] = 1;
  slaveData[1] = 255;

  digitalWrite(SS, LOW);
  SPI.transfer(slaveData, 5);
  digitalWrite(SS, HIGH);
  delay(1000);

  digitalWrite(SS, LOW);
  SPI.transfer(slaveData, 5);
  digitalWrite(SS, HIGH);
  delay(1000);

  slaveData[0] = 4;
}

void ScoringTable::onUpdate() {
  pollButtons();
  pollLights();
  delay(1000);
}
#endif