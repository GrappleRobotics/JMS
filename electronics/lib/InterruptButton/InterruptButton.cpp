#include <Arduino.h>
#include "InterruptButton.h"

InterruptButton::InterruptButton(const int intPin, void (*onInterrupt)(void)) : _interruptPin(intPin) {
  pinMode(_interruptPin, INPUT_PULLUP);
  attachInterrupt(digitalPinToInterrupt(_interruptPin), onInterrupt, FALLING);
}