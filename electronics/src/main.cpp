#include <Arduino.h>

#include <CAN.h>
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
  Serial.begin(9600);

  if (!CAN.begin(8E6)) {
    Serial.println("CAN Startup failed");
  }
  // CAN.

  CAN.filter(0x7e8);

}

void loop() {
  
}