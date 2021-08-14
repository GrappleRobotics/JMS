#include "STC.h"

int STC_Controller::onInit() {
	Handle(
		std::cout << "STC Init" << std::endl;
		createNetwork(JMS_IP, JMS_PORT, JMS_BUFFER_SIZE);
		getNetwork().nt_init();

		/**
		 * Set initial send values
		 */
		std::cout << "Sending Role to JMS" << std::endl;
		this->getNetwork().getSendPacket()->role = jms_electronics_NodeRole::jms_electronics_NodeRole_NODE_SCORING_TABLE;
		// this->getNetwork().getSendPacket()->ipv4 = 


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