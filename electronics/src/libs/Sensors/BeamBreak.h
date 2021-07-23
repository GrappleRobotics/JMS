#ifndef BEAMBREAK_H
#define BEAMBREAK_H
#include <mbed.h>
#include <iostream>

/**
 * State class for Beam State
 */
enum class BeamState {
	BROKEN = 0,
	UNBROKEN = 1
};

/**
 * Beam break class for 2 state beam breakers
 */
class BeamBreak {
 public:
	BeamBreak(PinName analogPin) {
		_beamBreak = new AnalogIn(analogPin);
	}

	~BeamBreak() {
		_beamBreak = NULL;
		delete _beamBreak;
	}

	/**
	 * If beam is broken returns true once. Does not continue to return true
	 */
	bool broke() {
		if (_state == BeamState::BROKEN && _prevState == BeamState::UNBROKEN) {
			return true;
		} else {
			return false;
		}
	}

	/**
	 * If beam is broken, continues to return true
	 */
	bool isBroken() {
		return _state == BeamState::BROKEN ? true : false;
	}

	/**
	 * Update the state stream (must be before any logic for beambreak)
	 */
	void updateStart() {
		updateState();
	}

	/**
	 * Update the state stream (must be after all the beambreak logic)
	 */
	void updateEnd() {
		updatePrevState();
	}

	std::pair<BeamState, BeamState> getStates() {
		return { _state, _prevState };
	}

	/**
	 * Set threshold for beambreak values between min and max thresh
	 */
	void setThreshold(int min, int max) {
		_minThresh = min;
		_maxThresh = max;
	}

	/**
	 * Set the reliability of the sensor.
	 * The higher the number, the more cycles it will take to determin if beam has been broken (Slower but more reliable)
	 * The lower the number, the less cycles it will use to determin. (Faster but less reliable)
	 * 
	 * Default is 3
	 */
	void setReliability(int rel) {
		_cycleBreak = rel;
	} 

 private:
	AnalogIn *_beamBreak;
	BeamState _state{ BeamState::UNBROKEN };
	BeamState _prevState{ BeamState::UNBROKEN };

	int _minThresh = 10;
	int _maxThresh = 1000;

	int _threshCycle = 0;
	int _cycleBreak = 3;

	void updateState() {
		if (_beamBreak->read_u16() > _minThresh && _beamBreak->read_u16() < _maxThresh) {
			_threshCycle++;
			
			if (_threshCycle > _cycleBreak) {
				_state = BeamState::BROKEN;
				_threshCycle = 0;
			}

		} else {
			_threshCycle = 0;
			_state = BeamState::UNBROKEN;
		}
	}

	void updatePrevState() {
		_prevState = _state;
	}
};

#endif