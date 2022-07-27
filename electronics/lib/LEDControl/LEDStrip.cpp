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