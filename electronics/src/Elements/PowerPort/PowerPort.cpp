#include "Elements/PowerPort/PowerPort.h"

int PowerPort::init(int argc, char const *argv[]) {
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

int PowerPort::update(int argc, char const *argv[]) {
	Handle(
		std::cout << "Test" << std::endl;
	)
}