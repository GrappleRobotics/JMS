// #include <mbed.h>
// #include <EthernetInterface.h>
// #include <FlashIAP.h>
// #include <thread>

// #ifndef NETWORK_BUF_SIZE
// #define NETWORK_BUF_SIZE 256
// #endif

// DigitalOut led(PB_0);

// /**
//  * Network controller test
//  */
// class Network {
//  public:

// 	Network(PinName pin = PB_0) {
// 		eth.connect();
// 		eth.get_ip_address(&s);
// 		printf("Server IP: %s\n", s.get_ip_address());

// 		// Server
// 		srv.open(&eth);
// 		srv.bind(9999); // Port bind
// 		srv.listen(1);
// 	}

// 	void update() {
// 		client = srv.accept();
// 		while (client != nullptr && client->recv(buffer, NETWORK_BUF_SIZE) > 0) {
// 			led = (atoi(buffer) != 0);
// 		}

// 		thread_sleep_for(50);
// 	}

//  private:
// 	EthernetInterface eth;
// 	SocketAddress s;
// 	TCPSocket srv, *client;
// 	char buffer[NETWORK_BUF_SIZE];
// };