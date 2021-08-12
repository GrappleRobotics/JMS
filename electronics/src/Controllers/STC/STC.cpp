#include "STC.h"

int STC_Controller::onInit() {
	Handle(
		std::cout << "STC Init" << std::endl;
		createNetwork(JMS_IP, JMS_PORT, JMS_BUFFER_SIZE);
		getNetwork().nt_init();

		// getStateController().setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND);
		// getNetwork().setNetwork(Network::State::NETWORK_SEND, "Test");
		// getNetwork().update();
	)
}

int STC_Controller::onUpdate() {
	Handle(
		// std::cout << "Type: " << (int)abortButton.getType() << std::endl;
		// @TODO Lighting magic
		// std::cout << "Test Loop" << std::endl;
	)
}