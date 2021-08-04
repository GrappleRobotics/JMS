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
		NETWORK_DO
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
		 */
		void setController(State st, Network::State nt_st = Network::State::IDLE, char *buffer = {0}) {
			if (_network != nullptr) {
				_state = st;
				_network->setNetwork(nt_st, "Hello");
				std::cout << "Network state: " << (int)_network->getState() << std::endl;
			} else {
				std::cout << "Network is null for state controller" << std::endl;
			}
		}
	 private:
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
			switch (_stateController->getState()) {
				case State::IDLE:
					break;

				case State::PROGRAM_DO:
					onUpdate();
					break;

				case State::NETWORK_DO:
					std::cout << "Networking..." << std::endl;
					std::cout << (int)_nt->getState() << std::endl;
					_nt->update();
					_stateController->setState(State::PROGRAM_DO);
					break;
			}
			ThisThread::sleep_for(100ms);
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