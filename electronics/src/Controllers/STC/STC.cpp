#include "STC.h"

int STC_Controller::onInit() {
	Handle(
		std::cout << "STC Init" << std::endl;
		createNetwork(JMS_IP, JMS_PORT, JMS_BUFFER_SIZE);
		getNetwork().nt_init();

		/**
		 * Set initial send values
		 */
		this->getNetwork().getSendPacket()->role = jms_electronics_NodeRole::jms_electronics_NodeRole_NODE_SCORING_TABLE;
		// this->getNetwork().getSendPacket()->ipv4 = 
		Abort abort({ABORT_1, ABORT_2, USER_BUTTON}, getStateController());


		/**
		 * Initial set of controller and update network
		 */
		getStateController().setController(MainController::State::PROGRAM_DO, Network::State::NETWORK_SEND);
		getNetwork().update();
	)
}



int STC_Controller::onUpdate() {
	Handle(
		/**
		 * Receive from server and do code
		 */
		
	)
}