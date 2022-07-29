#ifndef COMMS_H
#define COMMS_H

#define k1Mbs 1000E3
#define k500Kbs 500E3
#define k250Kbs 250E3
#define k200Kbs 200E3
#define k125Kbs 125E3
#define k100Kbs 100E3
#define k80Kbs 80E3
#define k50Kbs 50E3
#define k40Kbs 40E3
#define K20Kbs 20E3
#define k10Kbs 10E3
#define k5Kbs 5E3

#include <CAN.h>
#include "DataPacket.h"

namespace Comms {

  class Comm {
   public:
    static void start() {
      CAN.begin(_baudRate);
    }

    static void stop() {
      CAN.end();
    }

    static void setBaudRate(long br) {
      _baudRate = br;
    }

    static void setNodeID(Message::Common::Device::Type t, int id = (int)Message::Common::Device::Type::kOther) {
      _device.setType(t, id);
    }

    template <typename NODE_T>
    static int sendDataTo(NODE_T n) {
      MsgPack::Packer packer;
      packer.serialize(n);

      int n_packets = (1 + ((packer.size() - 1) / 8)); // get number of packets to store message

      // Start header packet
      // if (!CAN.available()) return 1;
      if (!CAN.beginExtendedPacket(n.device.__id)) return -1;
      CAN.write(n_packets);
      CAN.write(packer.size());
      if (!CAN.endPacket(1000)) return -1;

      int index = 0;
      for (byte i = 0; i < n_packets; i++) { // write packet

        if (CAN.beginExtendedPacket(n.device.__id)) return -1;
        for (size_t j = 0; j < 8; j++) { // write bytes to packet
          CAN.write(packer.data()[index]);
          index++;
        }
        if (CAN.endPacket(1000)) return -1;
      }
      return 0;
    }

    template <typename NODE_T>
    static NODE_T getData(NODE_T default_node) {
      default_node.device.__emptyData = true;

      CAN.filterExtended(_device.__id);

      int message_n_packets = 0; // message size in packets (each packet holds 8 bytes)
      int message_size = 0; // size of message in bytes

      // Check if data is being send
      int packetSize = CAN.parsePacket();
      if (packetSize <= 0) return default_node;

      // Read header
      if (!CAN.packetRtr()) {
        if (CAN.available()) {
          message_n_packets = CAN.read();
          message_size = CAN.read();
        }

        if (CAN.available() || message_n_packets < 0 || message_size < 0) {
          Serial.println("Bad header read");
          return default_node;
        }
      }

      // Read message
      byte messageBuffer[message_size];
      int counter = 0;
      for (int i = 0; i < message_n_packets; i++) {
        packetSize = CAN.parsePacket(); // between 1-8 bytes
        while (CAN.available()) {
          int buf_byte = CAN.read();
          if (buf_byte < 0) return default_node;
          messageBuffer[counter] = buf_byte;
          counter++;
        }
      }

      MsgPack::Unpacker unpacker;
      unpacker.feed(messageBuffer, message_size);
      NODE_T data;
      unpacker.deserialize(data);
      return data;
    }

   private:
    static long _baudRate;
    static Message::Common::Device _device;
  };
}

#endif