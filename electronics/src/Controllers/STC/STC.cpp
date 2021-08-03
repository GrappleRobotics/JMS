#include "STC.h"

const char *ECHO_SERVER_ADDRESS = "192.168.178.125";
const int ECHO_SERVER_PORT = 5800;
EthernetInterface eth;

int STC_Controller::start(int argc, char const *argv[]) {

	Handle(
		setRUNNING(true);
		char buf[1024] = "Hello world";

		// Network testing
		eth.connect();

		SocketAddress a;
		eth.get_ip_address(&a);

		// connect to server
		TCPSocket socket;
		
		if (socket.open(&eth) < 0) {
			std::cout << "Failed to open TCP socket" << std::endl;
		}

		eth.gethostbyname("ifconfig.io", &a);
		a.set_ip_address(ECHO_SERVER_ADDRESS);
		a.set_port(ECHO_SERVER_PORT);

		std::cout << "Client IP Addr: " << a.get_ip_address() << std::endl;
		std::cout << "Client Port: " << a.get_port() << std::endl;

		while (socket.connect(a) < 0) {
			std::cout << "Unable to connect to " << ECHO_SERVER_ADDRESS << ":" << ECHO_SERVER_PORT << std::endl;
		}

		std::cout << "Connected to server at: " << ECHO_SERVER_ADDRESS << ":" << ECHO_SERVER_PORT << std::endl;

		socket.send(buf, sizeof(buf) - 1);

		/**
		 * Scoring table Abort Button
		 */
		#ifdef STM
		Abort abort({ABORT_1, ABORT_2});
		#endif

		LoopHandle(getRUNNING(),
			// @TODO (lighting garbage)

			// Networking
			
		)
	)
}