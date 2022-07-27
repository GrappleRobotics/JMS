#include <Arduino.h>
#include "LEDStrips.h"

using namespace LED;

void Strip::setBrightness(byte value) {
  FastLED.setBrightness(value);
}

void Strip::set(unsigned int index, CRGB rgb, bool noShow) {
  _strip[index] = rgb;
  if (!noShow) FastLED.show();
}

void Strip::set(CRGB rgb, bool noShow) {
  for (size_t i = 0; i < _size; i++) {
    set(i, rgb, true);
  }
  
  if (!noShow) FastLED.show();
}

void Strip::setRainbow(int speed, bool noShow) {
  for (int j = 0; j < 255; j++) {
    for (size_t i = 0; i < _size; i++) {
      _strip[i] = CHSV(i - (j*2), 255, 255);
    }
    if (!noShow) FastLED.show();
    delay(speed);
  }
}

void Strip::setWave(CRGB colour, int waveSize, int speed, bool noShow) {
  for (size_t i = 0; i < _size; i++) {
    _strip[i-waveSize].setRGB(0,0,0);
    _strip[i] = colour;
    delay(speed);
    if (!noShow) FastLED.show();
  }
  FastLED.clear();
  if (!noShow) FastLED.show();
}

void Strip::setPulse(CRGB colour, int speed, bool noShow) {
  for (byte i = 255; i > 0; i--) {
    setBrightness(i);
    set(colour);
    if (!noShow) FastLED.show();
    delay(speed);
  }

  for (byte i = 0; i < 255; i++) {
    setBrightness(i);
    set(colour);
    if (!noShow) FastLED.show();
    delay(speed);
  }
}