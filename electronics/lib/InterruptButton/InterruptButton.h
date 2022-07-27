#ifndef INTERRUPT_BUTTON_H
#define INTERRUPT_BUTTON_H

enum class InterruptState {
  IDLE = 0,
  TRIGGERED = 1
};

/**
 * @brief Interrupt button
 * 
 */
class InterruptButton {
 public:
  /**
   * @brief Construct a new Interrupt Button object
   * 
   * @param intPin 
   * @param onInterrupt 
   * @param pullup 
   * @param mode 
   */
  InterruptButton(const int intPin, void (*onInterrupt)(void), bool pullup = false, int mode = 1);

  /**
   * @brief Get the Pin object
   * 
   * @return const int 
   */
  const int getPin() {
    return _interruptPin;
  }

  /**
   * @brief Set the Interrupt Mode object
   * 
   * @param mode 
   */
  void setInterruptMode(int mode);

  /**
   * @brief Update state (called automatically during getters)
   * 
   */
  void update();

  /**
   * @brief Get the State of the interrupt
   * 
   * @return InterruptState 
   */
  InterruptState getState() {
    update();
    return _state;
  }

  /**
   * @brief Get the state of the interrupt, boolean
   * 
   * @return true 
   * @return false 
   */
  bool isTriggered() {
    update();
    return _state == InterruptState::TRIGGERED ? true : false;
  }

 private:
  void (*_onInterrupt)(void);
  InterruptState _state{ InterruptState::IDLE };
  const int _interruptPin;
};

#endif