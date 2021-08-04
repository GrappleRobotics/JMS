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
class STC_Controller : public MainController::Controller {
 public:
	STC_Controller() {
		std::cout << "STC Mode" << std::endl;
	}

	// main controller functions
	int onInit() override;
	int onUpdate() override;
};

#endif // STC_H