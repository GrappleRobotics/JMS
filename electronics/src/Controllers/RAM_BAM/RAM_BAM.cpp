#include "Controllers/RAM_BAM/RAM_BAM.h"
#include "libs/E_Stop/E_Stop.h"

int RAM_BAM::start(int argc, char const *argv[], int &userButton) {
	Handle(
		setRUNNING(true);

		/**
		 * Alliance station E-Stop buttons
		 */
		#ifdef RAM
		E_Stop eStop1(E_STOP1_1, E_STOP1_2, E_StopType::E_STOP_R_ALLIANCE_1);
		E_Stop eStop2(E_STOP2_1, E_STOP2_2, E_StopType::E_STOP_R_ALLIANCE_2);
		E_Stop eStop3(E_STOP3_1, E_STOP3_2, E_StopType::E_STOP_R_ALLIANCE_3);
		#endif

		#ifdef BAM
		E_Stop eStop1(E_STOP1, E_StopType::E_STOP_B_ALLIANCE_1);
		E_Stop eStop2(E_STOP2, E_StopType::E_STOP_B_ALLIANCE_2);
		E_Stop eStop3(E_STOP3, E_StopType::E_STOP_B_ALLIANCE_3);
		#endif

		// PowerPort pp;
		// HandleElement(pp.init(argc, argv, userButton));
		
		LoopHandle(getRUNNING(),
			// HandleElement(pp.update(argc, argv, userButton));
			wait_us(100000);
		)
	)
}