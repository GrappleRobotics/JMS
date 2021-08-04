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
	 */
	void checkConn() {
		if (_local_socket.connect(_remote_address) != NSAPI_ERROR_IS_CONNECTED) {
			std::cout << "ERR: Connection lost, re-connecting...." << std::endl;
			_local_socket.close();
			nt_init();
		}
	}

	/**
	 * Get the buffer size in bytes
	 */
	size_t getBufferSize() {
		return (sizeof(char) * _bufferSize - 1);
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
		_buffer = buffer;
	}

	char *update() {
		char *tmp = {0};
		switch (_state) {
			case State::UN_INITIALIZED:
				std::cout << "-- WARNING --" << std::endl;
				std::cout << "[Network Unintialized]" << std::endl;
				break;

			case State::IDLE:
				checkConn();
				break;
			
			case State::NETWORK_SEND:
				nt_send(_buffer);
				break;

			case State::NETWORK_RECV:
				tmp = nt_recv();
				break;
		}
		return tmp;
	}

 private: //// Private

	/**
	 * Sender and receivers
	 */
	void nt_send(char *buffer) { _local_socket.send(buffer, getBufferSize()); }
	char *nt_recv() {
		char *buffer;
		_local_socket.recv(buffer, getBufferSize());
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