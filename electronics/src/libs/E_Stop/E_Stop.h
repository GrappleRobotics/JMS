#ifndef ESTOP
#define ESTOP

#include <vector>
#include "libs/Button/Button.h"
#include "libs/Controller.h"

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
	E_Stop(PinName buttonPin, E_StopType type, MainController::StateController &ct) : _ct(ct) {
		this->_type = type;

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

		/**
		 * Push back interrupts
		 */
		for (size_t i = 0; i < buttonPin.size(); i++) {
			_int.push_back(ButtonInterrupt(buttonPin[i], callback(this, &E_Stop::sendStop)));
		}
	}

	/**
	 * Send stop signal/change current state to network send stop
	 */
	void sendStop() {
		char *test = "E Stop Works";
		switch (_type) {
			case E_StopType::NONE:
				_type = E_StopType::ABORT;
				sendStop();
				break;

			// Blue Alliance stations
			case E_StopType::E_STOP_B_ALLIANCE_1:
				_ct.setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, test);
				break;

			case E_StopType::E_STOP_B_ALLIANCE_2:
				_ct.setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, test);
				break;

			case E_StopType::E_STOP_B_ALLIANCE_3:
				_ct.setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, test);
				break;

			// Red Alliance stations
			case E_StopType::E_STOP_R_ALLIANCE_1:
				_ct.setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, test);
				break;

			case E_StopType::E_STOP_R_ALLIANCE_2:
				_ct.setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, test);
				break;

			case E_StopType::E_STOP_R_ALLIANCE_3:
				_ct.setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, test);
				break;

			// Abort game
			case E_StopType::ABORT:
				_ct.setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, test);
				break;
		}
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
	MainController::StateController &_ct;
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