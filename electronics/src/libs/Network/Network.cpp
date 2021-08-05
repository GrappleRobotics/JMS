#include "libs/Network/Network.h"

Network::Network(const char *ip, int port, const int bufferSize) : _bufferSize(bufferSize), _ip(ip), _port(port) {
	_eth.connect();
	_eth.get_ip_address(&_remote_address);
	std::cout << "Created Network Interface" << std::endl;
}

void Network::nt_init() {
	int connection = 1;

	// Buffer size malloc
	_buffer = (char *)malloc(sizeof(char)*getBufferSize());
	std::cout << "Buffer size set" << std::endl;

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
			programValue = nt_send(_buffer);
			if (programValue == 0) {
				// Flush local buffer
				_buffer = NULL;
				setState(State::IDLE);
			}
			break;

		case State::NETWORK_RECV:
			_buffer = NULL;
			// nt_recv();
			programValue = 0;
			setState(State::IDLE);
			break;
	}

	return programValue != 0 ? 1 : 0;
}

int Network::nt_send(char *buffer) {
	int bytes = _local_socket.send(buffer, getBufferSize());
	if (checkConn() != 0) {
		_local_socket.send(buffer, getBufferSize());
		return 1;
	} else {
		std::cout << "\nSent data: " << buffer << ", Bytes: " << bytes << std::endl;
		return 0;
	}
}

char *Network::nt_recv() {
	char *buffer;
	_local_socket.recv(buffer, getBufferSize());
	if (checkConn() != 0) {
		_local_socket.recv(buffer, getBufferSize());
	}
	return buffer;
}