#include "Elements/PowerPort/PowerPort.h"
#include <thread>

int PowerPort::_inner_bb_function() {
	Handle(
		_inner_bb->updateStart();
		if (_inner_bb->broke()) {
			_innerCounter++;
			std::cout << "Inner Counter: " << _innerCounter << std::endl;
		}
		_inner_bb->updateEnd();
	)
}

int PowerPort::_outer_bb_function() {
	Handle(
		_outer_bb->updateStart();
		if (_outer_bb->broke()) {
			_outerCounter++;
			std::cout << "Outer Counter: " << _outerCounter << std::endl;
		}
		_outer_bb->updateEnd();
	)
}

int PowerPort::_lower_bb_function() {
	Handle(
		_lower_bb->updateStart();
		if (_lower_bb->broke()) {
			_lowerCounter++;
			std::cout << "Lower Counter: " << _lowerCounter << std::endl;
		}
		_lower_bb->updateEnd();
	)
}

int PowerPort::init(int argc, char const *argv[], int &userButton) {
	Handle(
		_innerCounter = 0;
		_outerCounter = 0;
		_lowerCounter = 0;

		assert(_innerCounter == 0);
		assert(_outerCounter == 0);
		assert(_lowerCounter == 0);

		std::cout << "Power port reset" << std::endl;
	)
}

int PowerPort::update(int argc, char const *argv[], int &userButton) {
	Handle(
		if (userButton != 1) {
			/**
			 * Beam breaks
			 */
			_inner_bb_function();
			_outer_bb_function();
			_lower_bb_function();
			// std::cout << "Functions run" << std::endl;
		} else {
			std::cout << "Init called" << std::endl;
			init(argc, argv, userButton);
		}
	)
}