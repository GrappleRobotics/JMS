#ifndef STM_H
#define STM_H

#include <mbed.h>
#include "Config.h"
#include <PinNames.h>
#include <iostream>

#include "libs/Controller.h"
#include "libs/E_Stop/E_Stop.h"

#include "libs/Network/Network.h"


/**
 * Scoring Table microcontroller class
 */
class STM_Controller : public Controller {
 public:
	STM_Controller() {
		std::cout << "STM Mode" << std::endl;
	}

	// main controller functions
	int start(int argc, char const *argv[], int &userButton) override;
};

#endif // STM_H