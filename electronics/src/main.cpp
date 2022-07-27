#include <Arduino.h>

#include "LEDStrips.h"
#include "InterruptButton.h"

volatile bool state = false;
void onInterrupt() {
  // state = !state;
  digitalWrite(LED_BUILTIN, HIGH);
  state = true;
}

InterruptButton button(2, &onInterrupt);
LED::Strip strip;

CRGB leds[120];

void setup() {
  // FastLED.addLeds<WS2812, 7, GRB>(leds, 120).setCorrection(TypicalLEDStrip);
  // pinMode(LED_BUILTIN, OUTPUT);
  // button.setInterruptMode(LOW);
  strip.create<WS2812<7, GRB>>(120);
  // strip.setBrightness(255);
  Serial.begin(9600);
}

void loop() {
  // strip.set({255,0,0});
  // FastLED.clear();
  // strip.set(4, (255,0,0));
  // for (size_t i = 0; i < strip.getSize(); i++) {
  //   strip.set(i-1, CRGB(0,0,0));
  //   strip.set(i, CRGB(255,0,0));
  // }
  // strip.set(CRGB(255,0,0));
  // for (size_t i = 0; i < strip.getSize(); i++) {
  //   strip.set(i-1, CRGB(0,0,0));
  //   strip.set(i, CRGB(255,0,0));
  // }
  // FastLED.clear();

  strip.setWave(CRGB(255,0,0), 5, 25);
  strip.setRainbow(25);
  strip.setPulse(CRGB(255,0,0), 0);
}