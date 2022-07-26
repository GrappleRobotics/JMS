#ifndef INTERRUPT_BUTTON_H
#define INTERRUPT_BUTTON_H

enum class InterruptState {
  IDLE = 0,
  TRIGGERED = 1
};

class InterruptButton {
 public:
  InterruptButton(const int intPin, void (*onInterrupt)(void));

  const int getPin() {
    return _interruptPin;
  }

  InterruptState getState() {
    return _state;
  }

  bool isTriggered() {
    return _state == InterruptState::TRIGGERED ? true : false;
  }

 private:
  InterruptState _state{ InterruptState::IDLE };
  const int _interruptPin;
};

#endif