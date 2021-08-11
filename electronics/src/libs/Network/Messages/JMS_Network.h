#ifndef JMS_NETWORK_H
#define JMS_NETWORK_H

#include <mbed.h>
#include <stdio.h>

/**
 * pb
 */
#include <pb.h>
#include <pb_decode.h>
#include <pb_encode.h>

// Gets built from compile_proto.sh
#include "JMS_Messages/messages.h"

/**
 * Encoder and decoder for JMS network messages
 */
class JMS_NetworkMessages {
 public:

	jms_electronics_UpdateNode2Field *getSendPacket() {
		return &_send_packet;
	}

	jms_electronics_UpdateField2Node *getReceivePacket() {
		return &_receive_packet;
	}

	uint8_t *getEncodedSendMessage(uint8_t *buffer) {
		pb_ostream_t stream = pb_ostream_from_buffer(buffer, sizeof(buffer));
		bool status = pb_encode(&stream, jms_electronics_UpdateNode2Field_fields, &_send_packet);
		if (!status) {
			printf("Encoding Failed: %s\n", PB_GET_ERROR(&stream));
		}

		return buffer;
	}

	static bool callback(pb_istream_t *stream, uint8_t *buffer, size_t count) {
		FILE *file = (FILE*)stream->state;
		bool result;

		if (count == 0) {
			return true;
		}

		result = _socket->recv(buffer, count);

		if (result == 0) {
			stream->bytes_left = 0;
		}

		return result == count;
	} 

	jms_electronics_UpdateField2Node getDecodedReceiveMessage(TCPSocket *socket, size_t bufferSize) {
		jms_electronics_UpdateField2Node tmpInputMessage = jms_electronics_UpdateField2Node_init_zero;

		_socket = socket;
		int fd;
		pb_istream_t stream = {&callback, buffer, SIZE_MAX};

		return tmpInputMessage;
	}

 protected:

	/**
	 * packet containers
	 * 
	 * - field2node -> receiving packet
	 * - node2field -> sending packet
	 */
	jms_electronics_UpdateNode2Field _send_packet = jms_electronics_UpdateNode2Field_init_zero;
	jms_electronics_UpdateField2Node _receive_packet = jms_electronics_UpdateField2Node_init_zero;
	static TCPSocket *_socket;
};


#endif