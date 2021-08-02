#ifndef STC_H
#define STC_H

#include <mbed.h>
#include "Config.h"
#include <PinNames.h>
#include <iostream>

#include "libs/Controller.h"
#include "libs/E_Stop/E_Stop.h"

/**
 * Scoring Table microcontroller class
 */
class STC_Controller : public Controller {
 public:
	STC_Controller() {
		std::cout << "STM Mode" << std::endl;
	}

	// main controller functions
	int start(int argc, char const *argv[]) override;
};

#endif // STC_H