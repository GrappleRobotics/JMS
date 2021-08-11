#include "libs/Controller.h"
#include "libs/E_Stop/E_StopType.h"

/**
 * 
 * 
 * -----------------------------------
 * Update status and controller states
 * -----------------------------------
 * 
 * 
 */


/**
 * E Stop
 * Sends e stop over the network
 */
void MainController::StateController::_estop() {
	switch (_inputFlag) {
		case (int)E_StopType::NONE:
			_intType = (int)E_StopType::ABORT;
			printf("None type Interrupt detected");
			break;

		// Blue Alliance stations
		case (int)E_StopType::E_STOP_B_ALLIANCE_1:
			printf("Alliance 1 Stop called");
			break;

		case (int)E_StopType::E_STOP_B_ALLIANCE_2:
			printf("Alliance 2 Stop called");
			break;

		case (int)E_StopType::E_STOP_B_ALLIANCE_3:
			printf("Alliance 3 Stop called");
			break;

		// Red Alliance stations
		case (int)E_StopType::E_STOP_R_ALLIANCE_1:
			printf("Alliance Stop called");
			break;

		case (int)E_StopType::E_STOP_R_ALLIANCE_2:
			printf("Alliance Stop called");
			break;

		case (int)E_StopType::E_STOP_R_ALLIANCE_3:
			printf("Alliance Stop called");
			break;

		// Abort game
		case (int)E_StopType::ABORT:
			printf("Abort called");
			break;
	}
}

void MainController::StateController::updateStatus() {
	if (_interruptFlag != 0) { // If flagged, then process interrupt
		switch (_intType) {
			case (int)InterruptType::NONE:
				break;

			case (int)InterruptType::E_STOP:
				_estop();
				break;
			
			default:
				_intType = (int)InterruptType::NONE;
				break;
		}
	}
}