#ifndef POWERPORT_H
#define POWERPORT_H

#include <mbed.h>
#include "Config.h"
#include <PinNames.h>
#include <iostream>
#include <thread>

#include "libs/Sensors/BeamBreak.h"
#include "libs/Element.h"

/**
 * Power port for 2020/2021 FRC game Infinite Recharge:
 * 3 Counters using beambreak sensors
 * 
 */
class PowerPort : public Element {
 public:
	PowerPort(PinName innerPort, PinName outerPort, PinName lowerPort) {
		_inner_bb = new BeamBreak(innerPort);
		_outer_bb = new BeamBreak(outerPort);
		_lower_bb = new BeamBreak(lowerPort);
		std::cout << "Power Port created" << std::endl;
	}

	// Main controlled functions
	int init(int argc, char const *argv[], int &userButton) override;
	int update(int argc, char const *argv[], int &userButton) override;

 private:
	BeamBreak *_inner_bb; // Beam breaks for power ports
	BeamBreak *_outer_bb;
	BeamBreak *_lower_bb;

	int _inner_bb_function();
	int _outer_bb_function();
	int _lower_bb_function();

	int _innerCounter = 0, _outerCounter = 0, _lowerCounter = 0;
};

#endif