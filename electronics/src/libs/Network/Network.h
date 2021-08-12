#ifndef NETWORK_H
#define NETWORK_H

/**
 * Config
 */
#include "Config.h"

/**
 * JMS Packets/Messages
 * Send/Recv
 */
#include "Messages/JMS_Network.h"


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
#include <iostream>


/**
 * 
 * Network Client for connecting to servers (JMS for the moment) 
 * @TODO Have server as optional setup in network
 */
class Network : public JMS_NetworkMessages {
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
	Network(const char *ip = JMS_IP, int port = JMS_PORT, const int bufferSize = JMS_BUFFER_SIZE);
	~Network() {
		printf("Yo, that shit just go detroyed [NETWORK]");
	}

	/**
	 * Initialize network and connect to server
	 */
	void nt_init();


	/**
	 * Check if still connected, and reconnect if not
	 * If connected returns 0, else returns 1
	 */
	int checkConn();

	/**
	 * Get the buffer size in bytes
	 */
	size_t getBufferSize() {
		return (sizeof(uint8_t) * _bufferSize);
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
	void setNetwork(State st) {
		setState(st);
	}

	/**
	 * Update network (State machine [send/recv])
	 */
	int update();

 private: //// Private
	/**
	 * Sender and receivers
	 */
	int nt_send();
	void nt_recv();

	/**
	 * Raw sender and receivers for buffers
	 */
	int nt_raw_send(uint8_t *buffer);
	uint8_t *nt_raw_recv();

	/**
	 * Network state
	 */
	State _state{ State::UN_INITIALIZED };

	/**
	 * Network packet buffer
	 */
	// uint8_t *_buffer;

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