#include "Controllers/RAM_BAM/RAM_BAM.h"
#include "libs/E_Stop/E_Stop.h"

int RAM_BAM::start(int argc, char const *argv[], int &userButton) {
	E_Stop(PC_13, E_StopType::ABORT);

	while(1) {
		printf("Looping In Controller\n");
		// wait_us(10000);
	}

	Handle(
		setRUNNING(true);

		/**
		 * Alliance station E-Stop buttons
		 */
		// printf("Creating instance of estop\n");
		// E_Stop estop(PC_13, E_StopType::ABORT);
		// printf("Instance created\n");
		#ifdef RAM
		// E_Stop eStop1(E_STOP1, E_StopType::E_STOP_R_ALLIANCE_1);
		// E_Stop eStop2(E_STOP2, E_StopType::E_STOP_R_ALLIANCE_2);
		// E_Stop eStop3(E_STOP3, E_StopType::E_STOP_R_ALLIANCE_3);
		#endif

		#ifdef BAM
		// E_Stop eStop1(E_STOP1, E_StopType::E_STOP_B_ALLIANCE_1);
		// E_Stop eStop2(E_STOP2, E_StopType::E_STOP_B_ALLIANCE_2);
		// E_Stop eStop3(E_STOP3, E_StopType::E_STOP_B_ALLIANCE_3);
		#endif

		PowerPort pp;
		HandleElement(pp.init(argc, argv, userButton));
		
		LoopHandle(getRUNNING(),
			// thread_sleep_for(5);
			HandleElement(pp.update(argc, argv, userButton));
			wait_us(100000);
			// printf("Looping\n");
		)
	)
}