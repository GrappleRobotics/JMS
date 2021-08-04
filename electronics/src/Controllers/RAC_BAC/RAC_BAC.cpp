#include "Controllers/RAC_BAC/RAC_BAC.h"

// int RAC_BAC_Controller::start(int argc, char const *argv[]) {
// 	Handle(
// 		setRUNNING(true);

// 		/**
// 		 * Alliance station E-Stop buttons
// 		 */
// 		#ifdef RAC
// 		E_Stop eStop1({E_STOP1_1, E_STOP1_2}, E_StopType::E_STOP_R_ALLIANCE_1);
// 		E_Stop eStop2({E_STOP2_1, E_STOP2_2}, E_StopType::E_STOP_R_ALLIANCE_2);
// 		E_Stop eStop3({E_STOP3_1, E_STOP3_2}, E_StopType::E_STOP_R_ALLIANCE_3);
// 		#endif

// 		#ifdef BAC
// 		E_Stop eStop1({E_STOP1_1, E_STOP1_2}, E_StopType::E_STOP_B_ALLIANCE_1);
// 		E_Stop eStop2({E_STOP2_1, E_STOP2_2}, E_StopType::E_STOP_B_ALLIANCE_2);
// 		E_Stop eStop3({E_STOP3_1, E_STOP3_2}, E_StopType::E_STOP_B_ALLIANCE_3);
// 		#endif

// 		// PowerPort pp;
// 		// HandleElement(pp.init(argc, argv, userButton));
		
// 		LoopHandle(getRUNNING(),
// 			// HandleElement(pp.update(argc, argv, userButton));
// 			wait_us(100000);
// 		)
// 	)
// }