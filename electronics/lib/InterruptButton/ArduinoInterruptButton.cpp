#ifdef ARDUINO
#include <Arduino.h>
#include "InterruptButton.h"

InterruptButton::InterruptButton(const int pin, void (*onInterrupt)(void), bool pullup) : _pin(pin) {
  if (pullup) {
    pinMode(pin, INPUT_PULLUP);
  } else {
    pinMode(pin, INPUT);
  }

  attachInterrupt(digitalPinToInterrupt(pin), onInterrupt, CHANGE);
  _intAttached = true;
}

InterruptButton::InterruptButton(const int pin, bool pullup) : _pin(pin) {
  if (pullup) {
    pinMode(pin, INPUT_PULLUP);
  } else {
    pinMode(pin, INPUT);
  }
}

void InterruptButton::attachInterruptFunction(void (*onInterrupt)(void)) {
  if (!_intAttached) {
    attachInterrupt(digitalPinToInterrupt(_pin), onInterrupt, FALLING);
  }
}

bool InterruptButton::isTriggered() {
  const int state = digitalRead(_pin);
  return state == HIGH ? false : true;
}

#endif