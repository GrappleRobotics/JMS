#include "STC.h"

int STC_Controller::start(int argc, char const *argv[]) {
	Handle(
		setRUNNING(true);

		/**
		 * Scoring table Abort Button
		 */
		#ifdef STM
		Abort abort({ABORT_1, ABORT_2});
		#endif

		LoopHandle(getRUNNING(),
			// @TODO (lighting garbage)
		)
	)
}