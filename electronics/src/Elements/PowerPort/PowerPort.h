#ifndef POWERPORT_H
#define POWERPORT_H

#include <mbed.h>
#include "Config.h"
#include <PinNames.h>
#include <iostream>

#include "libs/Sensors/BeamBreak.h"
#include "libs/Controller.h"

/**
 * Power port for 2020/2021 FRC game Infinite Recharge:
 * 3 Counters using beambreak sensors
 * 
 */
class PowerPort : public Controller {
 public:
	PowerPort() {
		std::cout << "Power Port created: ";
		if (MODE == 0) {
			std::cout << "RAM Mode" << std::endl;
		} else if (MODE == 1) {
			std::cout << "BAM Mode" << std::endl;
		}
	}

	// Main controlled functions
	int init(int argc, char const *argv[]) override;
	int update(int argc, char const *argv[]) override;

 private:
	BeamBreak _inner_bb{ INNER_BB_PORT }; // Beam breaks for power ports
	BeamBreak _outer_bb{ OUTER_BB_PORT };
	BeamBreak _LOWER_bb{ LOWER_BB_PORT };

	int _innerCounter = 0, _outerCounter = 0, _lowerCounter = 0;
};

#endif