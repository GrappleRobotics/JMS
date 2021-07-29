#ifndef ESTOP
#define ESTOP

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
class E_Stop : public ButtonInterrupt {
 public:
	E_Stop(PinName buttonPin, E_StopType type) : ButtonInterrupt(buttonPin, callback(this, &E_Stop::sendStop)) {
		printf("E Stop created\n");
		this->_type = type;
	}

	void sendStop() {
		// @TODO
		printf("E Stop triggered");
	}

 private:
	E_StopType _type;
};

/**
 * Abort, super version of E-Stop, sends abort signal
 */
class Abort : public E_Stop {
 public:
	Abort(PinName buttonPin) : E_Stop(buttonPin, E_StopType::ABORT) {}
};



#endif