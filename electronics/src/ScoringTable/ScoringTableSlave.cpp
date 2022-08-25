#include "ScoringTable/ScoringTableSlave.h"
#if NODE_TYPE==3

#include <Arduino.h>
#include <SPI.h>
#include "Comms/comms.h"

ScoringTableSlave::ScoringTableSlave() : NodeBase(0,0) {}

uint8_t slaveData[5] = {0,0,0,0,0};
volatile uint8_t pos;
volatile bool process_it;

void ScoringTableSlave::init() {
  Serial.begin(115200);
  pinMode(MISO, OUTPUT);
  SPCR |= _BV(SPE);
  SPCR |= _BV(SPIE);

  pos = 0;
  process_it = false;

  _strip.create<WS2812<2, GRB>>(120); // 120 led strip
}

ISR(SPI_STC_vect) {
  uint8_t c = SPDR;

  if (pos < 5) {
    slaveData[pos++] = c;

    if (pos == 4) {
      process_it = true;
    }
  }

  Serial.println(slaveData[0]);
  Serial.println(slaveData[1]);
  Serial.println(slaveData[2]);
  Serial.println(slaveData[3]);
  Serial.println(slaveData[4]);
}

void ScoringTableSlave::updateLights() {
  switch (_mode) {
    case 0:
      _strip.set(CRGB(0,0,0));
      break;

    case 1:
      _strip.set(CRGB(_rgb[0], _rgb[1], _rgb[2]));
      break;

    case 2:
      _strip.setPulse(CRGB(_rgb[0], _rgb[1], _rgb[2]), _duration);
      break;

    case 3:
      _strip.setWave(CRGB(_rgb[0], _rgb[1], _rgb[2]), 5, _duration);
      break;

    case 4:
      _strip.setRainbow(_duration);
      break;

    default:
      _strip.set(CRGB(0,0,0));
      break;
  }
}

void ScoringTableSlave::onUpdate() {
  if (process_it) {
    _mode = slaveData[0];
    _rgb[0] = slaveData[1];
    _rgb[1] = slaveData[2];
    _rgb[2] = slaveData[3];
    _duration = slaveData[4];
    pos = 0;
    process_it = false;
  }

  updateLights();
}
#endif