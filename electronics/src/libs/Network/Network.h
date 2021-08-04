#ifndef NETWORK_H
#define NETWORK_H

/**
 * Config
 */
#include "Config.h"


/**
 * pb
 */
#include <pb.h>
#include <pb_decode.h>
#include <pb_encode.h>

/**
 * mbed socket
 */
#include <EthernetInterface.h>
#include <FlashIAP.h>


/**
 * Sys headers
 */
#include <vector>
#include <string.h>


/**
 * 
 * Network Client for connecting to servers (JMS for the moment) 
 * @TODO Have server as optional setup in network
 */
class Network {
 public:

	enum class State {
		UN_INITIALIZED = 0,
		IDLE,
		NETWORK_SEND,
		NETWORK_RECV
	};

	/**
	 * Create the interface and pass through the network values (IP/Port/buffSize)
	 */
	Network(const char *ip = JMS_IP, int port = JMS_PORT, const int bufferSize = JMS_BUFFER_SIZE) : _bufferSize(bufferSize), _ip(ip), _port(port) {
		_eth.connect();
		_eth.get_ip_address(&_remote_address);
		std::cout << "Created Network Interface" << std::endl;
	}

	/**
	 * Initialize network and connect to server
	 */
	void nt_init() {
		// Open socket
		std::cout << "Opening socket..." << std::endl;
		if (_local_socket.open(&_eth) < 0) {
			std::cout << "Failed to open TCP socket" << std::endl;
		}
		std::cout << "-- Socket open --" << std::endl;

		_remote_address.set_ip_address(_ip);
		_remote_address.set_port(_port);

		std::cout << "Connecting to server..." << std::endl;
		while (_local_socket.connect(_remote_address) < 0) {
			std::cout << "Unable to connect to " << _remote_address.get_ip_address() << ":" << _remote_address.get_port() << std::endl;
		}

		std::cout << "-- Connected to server at: " << _remote_address.get_ip_address() << ":" << _remote_address.get_port() << " --" << std::endl;
		setState(State::IDLE);
	}


	/**
	 * Check if still connected, and reconnect if not
	 * If connected returns 0, else returns 1
	 */
	int checkConn() {
		if (_local_socket.connect(_remote_address) != NSAPI_ERROR_IS_CONNECTED) {
			std::cout << "ERR: Connection lost, re-connecting...." << std::endl;
			_local_socket.close();
			nt_init();
			return 1;
		} else {
			return 0;
		}
	}

	/**
	 * Get the buffer size in bytes
	 */
	size_t getBufferSize() {
		return (sizeof(char) * _bufferSize);
	}

	/**
	 * Get current state of network
	 */
	State getState() {
		return _state;
	}

	/**
	 * Set the state of the network
	 */
	void setState(State st) {
		_state = st;
	}

	/**
	 * Set the network, 
	 * and on next update it will process and execute request
	 */
	void setNetwork(State st, char *buffer = {0}) {
		setState(st);
		_buffer = nullptr; // flush the local buffer
		_buffer = buffer;
	}

	int update() {
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
					setState(State::IDLE);
				}
				break;

			case State::NETWORK_RECV:
				// nt_recv();
				programValue = 0;
				setState(State::IDLE);
				break;
		}

		return programValue != 0 ? 1 : 0;
	}

 private: //// Private

	/**
	 * Sender and receivers
	 */
	int nt_send(char *buffer) {
		int bytes = _local_socket.send(buffer, getBufferSize());
		if (checkConn() != 0) {
			_local_socket.send(buffer, getBufferSize());
			return 1;
		} else {
			std::cout << "\nSent data: " << buffer << ", Bytes: " << bytes << std::endl;
			return 0;
		}
	}

	char *nt_recv() {
		char *buffer;
		_local_socket.recv(buffer, getBufferSize());
		if (checkConn() != 0) {
			_local_socket.recv(buffer, getBufferSize());
		}
		return buffer;
	}

	/**
	 * Network state
	 */
	State _state{ State::UN_INITIALIZED };

	/**
	 * Network packet
	 */
	char *_buffer;

	/**
	 * Network values
	 */
	const int _bufferSize;
	const char *_ip;
	const int _port;

	/**
	 * Socket values
	 */
	EthernetInterface _eth;
	TCPSocket _local_socket;
	SocketAddress _remote_address;
};

#endif