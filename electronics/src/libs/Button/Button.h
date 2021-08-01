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
		_passthrough = false;
	}

	Button(int *passthroughState) : _buttonState(passthroughState) {
		_button = nullptr;
		_passthrough = true;
		printf("Passthrough button created\n");
	}
	
	~Button() {
		_button = nullptr;
		_buttonState = nullptr;
		delete _button;
		delete _buttonState;
	}

	/**
	 * updates button state in seperate thread.
	 */
	void update() {
		if (_passthrough) { // passthrough check
			if (_buttonState) {
				_triggered = true;
				_currentlyTriggered = true;
			} else {
				_currentlyTriggered = false;
			}
		} else { // local check
			if (_button) {
				_triggered = true;
				_currentlyTriggered = true;
			} else {
				_currentlyTriggered = false;
			}
		}
	}

	/**
	 * Gets current state of button
	 */
	bool isTriggered() {
		update();
		return _currentlyTriggered;
	}

	/**
	 * Returns if the button has ever been triggered
	 */
	bool triggered() {
		update();
		return _triggered;
	}

 private:
	DigitalIn *_button; // used if regular button
	int *_buttonState; // used if passthrough button

	bool _passthrough = false;
	bool _triggered = false; // has button ever been triggered
	bool _currentlyTriggered = false;
};

/**
 * Main class for interrupt button,
 * Stores button current state and trigger events
 * also interrupt with function event
 */
class ButtonInterrupt {
 public:
	ButtonInterrupt(PinName interruptPin, Callback<void()> bindFunction, int startingState = 0) {
		_interruptButton = new InterruptIn(interruptPin);
		printf("Created new interrupt in\n");
		// *_interruptButtonState = *_interruptButton;
		// printf("Button satte is not button interrupt\n");
		_interruptButton->rise(bindFunction);
		printf("Button interrupt created\n");
	}

	~ButtonInterrupt() {
		_interruptButton = nullptr;
		delete _interruptButton;
	}

 private:
	int *_interruptButtonState = 0;
	InterruptIn *_interruptButton;
};

#endif