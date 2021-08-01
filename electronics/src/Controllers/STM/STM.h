#ifndef STM_H
#define STM_H

#include <mbed.h>
#include "Conifg.h"
#include <PinNames.h>
#include <iostream>

#include "libs/Controller.h"

/**
 * Scoring Table microcontroller class
 */
class STM : public Controller {
 public:
	STM() {
		std::cout << "STM Mode" << std::endl;
	}

	// main controller functions
	int start(int argc, char const *argv[], int &userButton) override;
};

#endif // STM_H