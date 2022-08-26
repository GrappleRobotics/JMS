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

  pinMode(A2, INPUT_PULLUP);
  pinMode(4, INPUT_PULLUP);
  pinMode(5, INPUT_PULLUP);
  pinMode(6, INPUT_PULLUP);
  pinMode(7, INPUT_PULLUP);
  pinMode(8, INPUT_PULLUP);
  pinMode(9, INPUT_PULLUP);
}

void ScoringTable::pollButtons() {
  const MessageEstops estop_message {
    EstopStates {
      // Field Estop
      !digitalRead(A2), 
      
      // Red Estops
      {
        !digitalRead(4),
        !digitalRead(5),
        !digitalRead(6)
      },

      // Blue Estops
      {
        !digitalRead(7),
        !digitalRead(8),
        !digitalRead(9)
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
}

void ScoringTable::onUpdate() {
  pollButtons();
  pollLights();
  delay(100);
}
#endif