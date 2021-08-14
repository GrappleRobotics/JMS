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
		void setController(State st, Network::State nt_st = Network::State::IDLE) {
			if (_network != nullptr) {
				_state = st;
				_network->setNetwork(nt_st);
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
			_interruptFlag = 1;
		}

		void resetPrimitiveInterruptValues() {
			_interruptFlag = 0;

			_intMain_st = 0;
			_intNt_st = 0;
			_intType = 0;
			_inputFlag = 0;
		}

		void updateStatus() {
			if (_interruptFlag != 0) {
				std::cout << "Do interrupt" << std::endl;
				switch (_intType) {
					case (int)InterruptType::NONE:
						std::cout << "No Interrupt type specified, or not supported yet" << std::endl;
						setController(State::PROGRAM_DO, Network::State::IDLE);
						_interruptFlag = 0;
						break;

					case (int)InterruptType::E_STOP:
						_estop(); // Configure E Stop type
						setController(State::INTERRUPT_DO, Network::State::NETWORK_SEND);
						_interruptFlag = 0;
						break;

					default:
						_intType = (int)InterruptType::NONE;
						break;
				}
			}
		}

	 private:
		int _estop();
		int _interruptFlag = 0;
		int _intMain_st;
		int _intNt_st;
		int _intType;
		int _inputFlag;
		State _state{ State::IDLE };
		Network *_network = nullptr;
	};

	/**
	 * Main Controller base class
	 * 
	 * @NOTE: Only the controller should be accessing the network, not sub layers e.g LED libraries.
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
				case State::IDLE: // do nothing
					break;

				case State::PROGRAM_DO: // receive from network, do code, then send
					onUpdate();
					break;

				case State::INTERRUPT_DO:
					std::cout << "Interrupting" << std::endl;
					if (_nt->update() == 0) {
						_stateController->setState(State::PROGRAM_DO);
					} else {
						std::cout << "Interrupt state error, trying again" << std::endl;
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