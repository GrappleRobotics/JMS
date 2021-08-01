#ifndef RAM_BAM_H
#define RAM_BAM_H

#include <mbed.h>
#include "Config.h"
#include <PinNames.h>
#include <iostream>

#include "libs/Controller.h"
#include "Elements/PowerPort/PowerPort.h"
#include "libs/E_Stop/E_Stop.h"

/**
 * Red alliance/Blue alliance microcontroller class
 */
class RAM_BAM_Controller : public Controller {
 public:
	RAM_BAM_Controller() {
		switch (MODE) {
			case 0:
				std::cout << "RAM Mode" << std::endl;
				break;
			case 1:
				std::cout << "BAM Mode" << std::endl;
				break;
		}
	}

	// Main controlled functions
	int start(int argc, char const *argv[], int &userButton) override;
};

#endif // RAM_BAM_H