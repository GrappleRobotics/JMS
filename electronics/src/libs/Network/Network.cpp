#include "libs/Network/Network.h"

Network::Network(const char *ip, int port, const int bufferSize) : _bufferSize(bufferSize), _ip(ip), _port(port) {
	_eth.connect();
	_eth.get_ip_address(&_remote_address);

	/**
	 * get ip and encode string
	 */
	this->getSendPacket()->ipv4.arg = (void *)_remote_address.get_ip_address();
	this->getSendPacket()->ipv4.funcs.encode = &encode_string;
	std::cout << "Created Network Interface" << std::endl;
}

void Network::nt_init() {

	// Open socket
	std::cout << "Opening socket..." << std::endl;
	if (_local_socket.open(&_eth) < 0) {
		std::cout << "Failed to open TCP socket" << std::endl;
	}
	std::cout << "-- Socket open --" << std::endl;

	_remote_address.set_ip_address(_ip);
	_remote_address.set_port(_port);

	std::cout << "Connecting to server..." << std::endl;
	while (_local_socket.connect(_remote_address) != 0) {
		std::cout << "Unable to connect to " << _remote_address.get_ip_address() << ":" << _remote_address.get_port() << std::endl;
	}

	if (_local_socket.connect(_remote_address) == NSAPI_ERROR_IS_CONNECTED) {
		std::cout << "-- Connected to server at: " << _remote_address.get_ip_address() << ":" << _remote_address.get_port() << " --" << std::endl;
	}
	setState(State::IDLE);
}


int Network::checkConn() {
	if (_local_socket.connect(_remote_address) != NSAPI_ERROR_IS_CONNECTED) {
		std::cout << "ERR: Connection lost, re-connecting...." << std::endl;
		_local_socket.close();
		nt_init();
		return 1;
	} else {
		return 0;
	}
}

int Network::update() {
	int programValue = 1;
	switch (_state) {
		case State::UN_INITIALIZED:
			std::cout << "-- WARNING --" << std::endl;
			std::cout << "[Network Unintialized]" << std::endl;
			break;

		case State::IDLE:
			programValue = checkConn();
			break;
		
		case State::NETWORK_SEND:
			programValue = nt_send();
			if (programValue == 0) {
				// Flush local buffer
				setState(State::IDLE);
			}
			break;

		case State::NETWORK_RECV:
			nt_recv();
			programValue = 0;
			setState(State::IDLE);
			break;
	}

	return programValue != 0 ? 1 : 0;
}

// Raw send of buffers
int Network::nt_raw_send(uint8_t *buffer) {
	int sendBytes = _local_socket.send(buffer, getBufferSize());
	if (sendBytes < 0) {
		printf("Send failed error: %d", sendBytes);
		return 1;
	} else {
		if (checkConn() != 0 ) {
			sendBytes = _local_socket.send(buffer, getBufferSize());
			if (sendBytes < 0) {
				printf("Send failed error: %d", sendBytes);
				return 1;
			}
			return 0;
		} 
		return 0;
	}
}

// Raw receive of buffer
uint8_t *Network::nt_raw_recv() {
	uint8_t *buffer = {};
	int recvBytes = _local_socket.recv(buffer, getBufferSize());
	if (recvBytes < 0) {
		printf("Receive failed error: %d", recvBytes);
	} else {
		if (checkConn() != 0) {
			recvBytes = _local_socket.recv(buffer, getBufferSize());
			if (recvBytes < 0) {
				printf("Receive failed error: %d", recvBytes);
			}
		}
	}
	return buffer;
}

// message send
int Network::nt_send() {
	uint8_t *buffer = encodeSendMessage();
	int sendBytes = _local_socket.send(buffer, getBufferSize());
	if (sendBytes < 0) {
		printf("Send failed error: %d", sendBytes);
		return 1;
	} else {
		if (checkConn() != 0) {
			sendBytes = _local_socket.send(buffer, getBufferSize());
			if (sendBytes < 0) {
				printf("Send failed error: %d", sendBytes);
				return 1;
			}
			return 0;
		}
		return 0;
	}
}

// message receive
void Network::nt_recv() {
	uint8_t *buffer = {};
	int recvBytes = _local_socket.recv(buffer, getBufferSize());
	if (recvBytes < 0) {
		printf("Receive failed error: %d", recvBytes);
	} else {
		if (checkConn() != 0) { // extra cheeky check to receive again if failed
			recvBytes = _local_socket.recv(buffer, getBufferSize());
			if (recvBytes < 0) {
				printf("Receive failed error: %d", recvBytes);
			}
		}

		// Decode
		decodeReceiveMessage(buffer, recvBytes);
	}
}