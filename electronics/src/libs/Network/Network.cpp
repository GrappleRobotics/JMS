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
	// Keep trying to open the socket until success
	while (_connStat != ConnectionStatus::CONNECTED) {
		if (_connStat == ConnectionStatus::CONNECTING) {
			std::cout << "Closing Socket, reconnecting..." << std::endl;
			_local_socket.close();
		}

		// Set the status to connecting
		_connStat = ConnectionStatus::CONNECTING;

		std::cout << "Opening socket..." << std::endl;
		if (_local_socket.open(&_eth) < 0) {
			std::cout << "Failed to open TCP socket" << std::endl;
		}
		std::cout << "-- Socket open --" << std::endl;

		_remote_address.set_ip_address(_ip);
		_remote_address.set_port(_port);

		std::cout << "Connecting to server..." << std::endl;
		if (_local_socket.connect(_remote_address) != 0) {
			std::cout << "[ERROR]: Unable to connect to " << _remote_address.get_ip_address() << ":" << _remote_address.get_port() << std::endl;
		}

		// check if it's connected
		if (_local_socket.connect(_remote_address) != NSAPI_ERROR_IS_CONNECTED) {
			std::cout << "[WARNING]: Binded but not connected to " << _remote_address.get_ip_address() << ":" << _remote_address.get_port() << std::endl;
		} else if (_local_socket.connect(_remote_address) == NSAPI_ERROR_IS_CONNECTED) {
			std::cout << "-- Connected to server at: " << _remote_address.get_ip_address() << ":" << _remote_address.get_port() << " --" << std::endl;
			_connStat = ConnectionStatus::CONNECTED;
		}
	}

	// Set the network state to idle, unless told otherwise
	setState(State::IDLE);
}


int Network::checkConn() {
	if (_local_socket.connect(_remote_address) != NSAPI_ERROR_IS_CONNECTED) {
		std::cout << "ERR: Connection lost, re-connecting...." << std::endl;
		_connStat = ConnectionStatus::CONNECTING;
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
				setState(State::IDLE);
			}
			break;

		case State::NETWORK_RECV:
			programValue = nt_recv();
			if (programValue == 0) {
				setState(State::IDLE);
			}
			break;

		default:
			setState(State::IDLE);
	}

	return programValue != 0 ? 1 : 0;
}

// Raw send of buffers
int Network::nt_raw_send(uint8_t *buffer) {
	checkConn();
	int sendBytes = _local_socket.send(buffer, getBufferSize());
	if (sendBytes < 0) {
		printf("Send first failed, error: %d\n", sendBytes);
		
		if (checkConn() != 1) {
			sendBytes = _local_socket.send(buffer, getBufferSize());
			if (sendBytes < 0) {
				printf("Send second failed, error: %d\n", sendBytes);
				return 1;
			}
			return 0;
		} else {
			return 1;
		}
	}
	return 0;
}

// Raw receive of buffer
uint8_t *Network::nt_raw_recv() {
	checkConn();
	uint8_t *buffer = {};
	int recvBytes = _local_socket.recv(buffer, getBufferSize());
	if (recvBytes < 0) {
		printf("Receive frist failed, error: %d\n", recvBytes);
		if (checkConn() != 1) {
			recvBytes = _local_socket.recv(buffer, getBufferSize());
			if (recvBytes < 0) {
				printf("Receive first failed, error: %d\n", recvBytes);
			}
		}
	}
	return buffer;
}

// message send
int Network::nt_send() {
	checkConn();
	uint8_t *buffer = encodeSendMessage(getBufferSize());
	printf("buffersize: %d\n", getBufferSize());
	int sendBytes = _local_socket.send(buffer, getBufferSize());
	if (sendBytes < 0) {
		printf("Send first failed, error: %d\n", sendBytes);
		if (checkConn() != 1) {
			sendBytes = _local_socket.send(buffer, getBufferSize());
			if (sendBytes < 0) {
				printf("Send second failed, error: %d\n", sendBytes);
				return 1;
			}
			return 0;
		} else {
			return 1;
		}
	}
	return 0;
}

// message receive/
int Network::nt_recv() {
	checkConn();
	uint8_t *buffer = {};
	int recvBytes = _local_socket.recv(buffer, getBufferSize());
	if (recvBytes < 0) {
		printf("Receive first failed, error: %d\n", recvBytes);
		if (checkConn() != 1) {
			recvBytes = _local_socket.recv(buffer, getBufferSize());
			if (recvBytes < 0) {
				printf("Receive second failed, error: %d\n", recvBytes);
				return 1;
			}
			return 0;
		} else {
			return 1;
		}
	}
	decodeReceiveMessage(buffer, recvBytes);
	return 0;
}