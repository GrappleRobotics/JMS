#include <mbed.h>

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
	BeamBreak(PinName digitalPin) {
		beamBreak = new DigitalIn(digitalPin);
	}

	~BeamBreak() {
		delete beamBreak;
	}

	/**
	 * Return stream of current beam state.
	 * continues to send 0 if beam is broken. And continues to send 1 if beam is unbroken
	 */
	BeamState getStateStream() {
		return state;
	}

	/**
	 * If beam is broken returns true once. Does not continue to return true
	 */
	bool broke() {
		if (state == BeamState::BROKEN && prevState == BeamState::UNBROKEN) {
			return true;
		} else {
			return false;
		}
	}

	/**
	 * If beam is broken, continues to return true
	 */
	bool isBroken() {
		return state == BeamState::BROKEN ? true : false;
	}

	void update() {
		if (beamBreak->read()) {
			state = BeamState::UNBROKEN;
		} else {
			state = BeamState::BROKEN;
		}

		prevState = state;
	}

 private:
	DigitalIn *beamBreak;
	BeamState state{ BeamState::UNBROKEN };
	BeamState prevState{ BeamState::UNBROKEN };
};