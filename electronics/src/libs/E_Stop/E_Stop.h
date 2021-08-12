#ifndef ESTOP
#define ESTOP

#include <vector>
#include "libs/Button/Button.h"
#include "libs/Controller.h"
#include "libs/E_Stop/E_StopType.h"


/**
 * E Stop class, uses Button Interrupt to perform logic and send data
 */
class E_Stop {
 public:

	/**
	 * Constructor (Single interrupt input)
	 */
	E_Stop(PinName buttonPin, E_StopType type, MainController::StateController &ct) : _ct(ct) {
		this->_type = type;
		this->_stopTypeInt = (int)_type;

		/**
		 * Push back interrupts
		 */
		_int.push_back(ButtonInterrupt(buttonPin, callback(this, &E_Stop::sendStop)));
	}

	/**
	 * Construcctor (specify all pins used for this interrupt)
	 */
	E_Stop(std::vector<PinName> buttonPin, E_StopType type, MainController::StateController &ct) : _ct(ct) {
		this->_type = type;
		this->_stopTypeInt = (int)_type;

		/**
		 * Push back interrupts
		 */
		for (size_t i = 0; i < buttonPin.size(); i++) {
			_int.push_back(ButtonInterrupt(buttonPin[i], callback(this, &E_Stop::sendStop)));
		}
	}

	E_StopType getType() {
		return _type;
	}

	/**
	 * Send stop signal/change current state to network send stop
	 */
	void sendStop() {
		_ct.interruptSetController((int)MainController::State::INTERRUPT_DO, (int)Network::State::NETWORK_SEND, (int)MainController::InterruptType::E_STOP, _stopTypeInt);
	}

	ButtonInterrupt get() {
		return _int[0];
	}

	std::vector<ButtonInterrupt> getVector() {
		return _int;
	}

 private:
	std::vector<ButtonInterrupt> _int;
	E_StopType _type;
	int _stopTypeInt;
	MainController::StateController &_ct;
	int _sendStopSent;
};

/**
 * Abort, superised version of E-Stop, sends abort signal
 */
class Abort : public E_Stop {
 public:
	Abort(PinName buttonPin, MainController::StateController &ct) : E_Stop(buttonPin, E_StopType::ABORT, ct) {}
	Abort(std::vector<PinName> buttonPin, MainController::StateController &ct) : E_Stop(buttonPin, E_StopType::ABORT, ct) {}
};



#endif