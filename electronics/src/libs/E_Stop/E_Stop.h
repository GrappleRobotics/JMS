#ifndef ESTOP
#define ESTOP

#include <vector>
#include "libs/Button/Button.h"

/**
 * E-Stop button type
 */
enum class E_StopType {
	NONE = 0,

	E_STOP_B_ALLIANCE_1,
	E_STOP_B_ALLIANCE_2,
	E_STOP_B_ALLIANCE_3,

	E_STOP_R_ALLIANCE_1,
	E_STOP_R_ALLIANCE_2,
	E_STOP_R_ALLIANCE_3,

	ABORT
};


/**
 * E Stop class, uses Button Interrupt to perform logic and send data
 */
class E_Stop {
 public:

	/**
	 * Constructor (Single interrupt input)
	 */
	E_Stop(PinName buttonPin, E_StopType type) {
		this->_type = type;

		/**
		 * Push back interrupts
		 */
		_int.push_back(ButtonInterrupt(buttonPin, callback(this, &E_Stop::sendStop)));
	}

	/**
	 * Construcctor (specify all pins used for this interrupt)
	 */
	E_Stop(std::vector<PinName> buttonPin, E_StopType type) {
		this->_type = type;

		/**
		 * Push back interrupts
		 */
		for (size_t i = 0; i < buttonPin.size(); i++) {
			_int.push_back(ButtonInterrupt(buttonPin[i], callback(this, &E_Stop::sendStop)));
		}
	}

	void sendStop() {
		// @TODO
		printf("E Stop triggered");
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
};

/**
 * Abort, superised version of E-Stop, sends abort signal
 */
class Abort : public E_Stop {
 public:
	Abort(PinName buttonPin) : E_Stop(buttonPin, E_StopType::ABORT) {}
	Abort(std::vector<PinName> buttonPin) : E_Stop(buttonPin, E_StopType::ABORT) {}
};



#endif