#ifndef CONTROLER_H
#define CONTROLER_H

#include <mbed.h>
#include <PinNames.h>
#include "libs/Network/Network.h"

namespace MainController {

	/**
	 * Current state container
	 */
	enum class State {
		IDLE = 0,
		PROGRAM_DO,
		INTERRUPT_DO
	};

	/**
	 * @TODO
	 * Might want to have different interrupt types for controller, not all have to be e stop types
	 * But for the moment it's just e stop
	 */
	enum class InterruptType {
		NONE = 0,
		E_STOP = 1
	};

	/**
	 * State controller
	 */
	class StateController {
	 public:
		void initNetwork(Network *nt) {
			_network = nt;
		}

		/**
		 * Get current state
		 */
		State getState() {
			return _state;
		}

		/**
		 * Set state
		 */
		void setState(State st) {
			_state = st;
		}

		/**
		 * Set controller
		 * Set the main state, the network state, and optionally the buffer
		 */
		void setController(State st, Network::State nt_st = Network::State::IDLE, uint8_t *buffer = {0}) {
			if (_network != nullptr) {
				_state = st;
				_network->setNetwork(nt_st, buffer);
			} else {
				std::cout << "Network is null for state controller" << std::endl;
			}
		}


		/**
		 * Set controller primitive, used for interrupts
		 * Sets the interrupt flag, next loop the program stops and completes interrupt code then continues
		 * 
		 * Set the main state of the program, the network state, interrupt type [NONE, E_STOP, PROGRAM], and input flag (if it's e stop, flag is e stop type)
		 * if it's program. it's @TODO/EXTRA
		 */
		void interruptSetController(int mainSt, int nt_st = 0, int intType = 0, int inputFlag = 0) {
			_intMain_st = mainSt;
			_intNt_st = nt_st;
			_intType = intType;
			_inputFlag = inputFlag;
			_interruptFlag++;
		}

		void updateStatus();

	 private:
		void _estop();
		int _interruptFlag = 0;
		int _intMain_st;
		int _intNt_st;
		uint8_t _intType;
		int _inputFlag;
		State _state{ State::IDLE };
		Network *_network = nullptr;
	};

	/**
	 * Main Controller base class
	 */
	class Controller {
	 public:
		Controller() {
			_stateController = new StateController();
		}
		virtual ~Controller() = default;

		/**
		 * User implemented init
		 */
		virtual int onInit() = 0;

		/**
		 * user implemented update
		 */
		virtual int onUpdate() = 0;

		/**
		 * Updater, state machine
		 */
		void update() {
			_stateController->updateStatus();
			switch (_stateController->getState()) {
				case State::IDLE:
					break;

				case State::PROGRAM_DO:
					onUpdate();
					break;

				case State::INTERRUPT_DO:
					if (_nt->update() == 0) {
						_stateController->setState(State::PROGRAM_DO);
					} else {
						printf("Interrupt state issue");
					}
					break;
			}
		}

		/**
		 * Get runner
		 */
		bool getRUNNING() {
			return _RUNNING;
		}

		/**
		 * Set running
		 */
		void setRUNNING(bool running) {
			_RUNNING = running;
		}

		/**
		 * Start and execute main controller code
		 */
		int start(int argc, char const *argv[]) {
			Handle(
				setRUNNING(true);
				_stateController->setState(State::PROGRAM_DO);
				onInit();
				LoopHandle(getRUNNING(),
					update();
				)
			)
		}

		/**
		 * Create the controller network with ip port and buffer size
		 */
		void createNetwork(const char *ip, int port, int bufferSize) {
			_nt = new Network(ip, port, bufferSize);
			_stateController->initNetwork(_nt);
		}

		/**
		 * Getter for network class (Access it's functions, init and send/recv data)
		 */
		Network &getNetwork() {
			return *_nt;
		}

		StateController &getStateController() {
			return *_stateController;
		}

	 private:
		bool _RUNNING = true;
		Network *_nt;
		StateController *_stateController;
	};

}

#endif