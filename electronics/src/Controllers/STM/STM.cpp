#include "STM.h"

int STM_Controller::start(int argc, char const *argv[], int &userButton) {
	Handle(
		setRUNNING(true);

		/**
		 * Scoring table Abort Button
		 */
		#ifdef STM
		Abort abort({ABORT_1, ABORT_2});
		#endif

		LoopHandle(getRUNNING(),
			std::cout << "Test" << std::endl;
			// @TODO (lighting garbage)
		)
	)
}