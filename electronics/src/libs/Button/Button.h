#ifndef BUTTON_H
#define BUTTON_H

#include <mbed.h>
#include <rtos.h>
#include <iostream>
#include <vector>

/**
 * Main class for button,
 * Stores button current state and trigger events
 */
class Button {
 public:
	Button(PinName digitalPin) {
		_button = new DigitalIn(digitalPin);
	}

	Button(PinName digitalPin, int &linkInterrupt);
	
	~Button() {
		_button = NULL;
		delete _button;
	}

	/**
	 * updates button state in seperate thread.
	 */
	void update() {
		// @TODO: check logic
		// @TODO: if triggered logic
	}

	/**
	 * Gets current state of button
	 */
	bool isTriggered() {}

	/**
	 * if trigger do logic (threaded)
	 */
	template<typename Logic>
	void ifTriggered(Logic triggerLogic) {
		function.push_back(triggerLogic);
	}

	/**
	 * Returns if the button has ever been triggered
	 */
	bool triggered() {}

	/**
	 * return true if button is pushed
	 */
	bool poll() {}


	/**
	 * Set button to state
	 */
	void setButton(int state) {}
 private:
	DigitalIn *_button;
	Thread _buttonChecker_t;
	std::vector<void (*)(void)> function;
};

#endif