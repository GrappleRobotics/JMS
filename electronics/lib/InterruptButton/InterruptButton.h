#pragma once

class InterruptButton {
 public:
  /**
   * @brief Construct a new Interrupt Button object
   * 
   * @param pin 
   * @param onInterrupt 
   * @param pullup 
   */
  InterruptButton(const int pin, void (*onInterrupt)(void), bool pullup);

  /**
   * @brief Construct a new Interrupt Button object
   * 
   * @param pin 
   * @param pullup 
   */
  InterruptButton(const int pin, bool pullup);

  /**
   * @brief attach interrupt if not done during construction
   * 
   * @param onInterrupt 
   */
  void attachInterruptFunction(void (*onInterrupt)(void));

  /**
   * @brief get button state
   * 
   * @return true 
   * @return false 
   */
  bool isTriggered();

 private:
  const int _pin;
  bool _intAttached = false;
};