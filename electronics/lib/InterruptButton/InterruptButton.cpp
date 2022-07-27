#include <Arduino.h>
#include "InterruptButton.h"

InterruptButton::InterruptButton(const int intPin, void (*onInterrupt)(void), bool pullup, int mode) : _onInterrupt(onInterrupt), _interruptPin(intPin) {
  if (pullup) {
    pinMode(_interruptPin, INPUT_PULLUP);
  } else {
    pinMode(_interruptPin, INPUT);
  }

  attachInterrupt(digitalPinToInterrupt(_interruptPin), _onInterrupt, FALLING);
}

void InterruptButton::setInterruptMode(int mode) {
  detachInterrupt(digitalPinToInterrupt(_interruptPin));
  attachInterrupt(digitalPinToInterrupt(_interruptPin), _onInterrupt, mode);
}

void InterruptButton::update() {
  const int state = digitalRead(_interruptPin);
  if (state == HIGH) {
    _state = InterruptState::TRIGGERED;
  } else {
    _state = InterruptState::IDLE;
  }
}