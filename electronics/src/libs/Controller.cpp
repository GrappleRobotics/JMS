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
int MainController::StateController::_estop() {
	int programValue = 1;
	switch (_inputFlag) {
		case (int)E_StopType::NONE:
			_intType = (int)E_StopType::ABORT;
			_network->getSendPacket()->data.scoringTable.abort = true;
			programValue = 0;
			printf("None type Interrupt detected. ABORTING");
			break;

		// Blue Alliance stations
		case (int)E_StopType::E_STOP_ALLIANCE_1:
			_network->getSendPacket()->data.alliance.estop1 = true;
			programValue = 0;
			printf("Alliance 1 Stop called");
			break;

		case (int)E_StopType::E_STOP_ALLIANCE_2:
			_network->getSendPacket()->data.alliance.estop2 = true;
			programValue = 0;
			printf("Alliance 2 Stop called");
			break;

		case (int)E_StopType::E_STOP_ALLIANCE_3:
			_network->getSendPacket()->data.alliance.estop3 = true;
			programValue = 0;
			printf("Alliance 3 Stop called");
			break;

		// Abort game
		case (int)E_StopType::ABORT:
			_network->getSendPacket()->data.scoringTable.abort = true;
			programValue = 0;
			printf("Abort called");
			break;

		default:
			_intType = (int)E_StopType::ABORT;
			printf("Entered default int state. Unknown error. Setting abort");
			break;
	}

	return programValue;
}