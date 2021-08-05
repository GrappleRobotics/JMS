#ifndef RAC_BAC_H
#define RAC_BAC_H

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
class RAC_BAC_Controller : public MainController::Controller {
 public:
	RAC_BAC_Controller() {
		switch (MODE) {
			case 0:
				std::cout << "RAC Mode" << std::endl;
				break;
			case 1:
				std::cout << "BAC Mode" << std::endl;
				break;
		}
	}

	// Main controlled functions
	// int start(int argc, char const *argv[]) override;
};

#endif // RAC_BAC_H