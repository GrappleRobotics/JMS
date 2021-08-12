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

	/**
	 * Encode the local send message.
	 * Returns the enocded buffer
	 */
	uint8_t *encodeSendMessage() {
		uint8_t *buffer;
		pb_ostream_t stream = pb_ostream_from_buffer(buffer, sizeof(buffer));

		bool status = pb_encode(&stream, jms_electronics_UpdateNode2Field_fields, &_send_packet);
		if (!status) {
			printf("Encoding Failed: %s\n", PB_GET_ERROR(&stream));
		}

		return buffer;
	}

	/**
	 * Decode form the buffer
	 * places decoded data into local receive message
	 */
	void decodeReceiveMessage(uint8_t *buffer, size_t message_length) {
		jms_electronics_UpdateField2Node tmpInputMessage = jms_electronics_UpdateField2Node_init_zero;

		pb_istream_t stream = pb_istream_from_buffer(buffer, message_length);
		bool status = pb_decode(&stream, jms_electronics_UpdateField2Node_fields, &tmpInputMessage);

		if (!status) {
			printf("Decoding failed: %s\n", PB_GET_ERROR(&stream));
		}

		_receive_packet = tmpInputMessage;
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