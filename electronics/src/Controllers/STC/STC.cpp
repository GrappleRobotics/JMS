#include "STC.h"

int STC_Controller::onInit() {
	Handle(
		std::cout << "STC Init" << std::endl;
		createNetwork(JMS_IP, JMS_PORT, JMS_BUFFER_SIZE);
		getNetwork().nt_init();

		#ifdef STC
		Abort abort({ABORT_1, ABORT_2, USER_BUTTON}, getStateController());
		#endif

		getStateController().setController(MainController::State::NETWORK_DO, Network::State::NETWORK_SEND, "Test From STC");
		// getNetwork().setNetwork(Network::State::NETWORK_SEND, "Test");
		// getNetwork().update();
	)
}

int STC_Controller::onUpdate() {
	Handle(
		// @TODO Lighting magic
		std::cout << "Test Loop" << std::endl;
	)
}