#ifndef CAN_COMMS_H
#define CAN_COMMS_H

// #define k1Mbs 1000E3
// #define k500Kbs 500E3
// #define k250Kbs 250E3
// #define k200Kbs 200E3
// #define k125Kbs 125E3
// #define k100Kbs 100E3
// #define k80Kbs 80E3
// #define k50Kbs 50E3
// #define k40Kbs 40E3
// #define K20Kbs 20E3
// #define k10Kbs 10E3
// #define k5Kbs 5E3

#include <CAN.h>
#include "Serializer/DataSerializer.h"

namespace Comms {
  class CANComm {
   public:
    static void start();
    static void stop();
    static void setBaudRate(long br);
    static void setID(unsigned int id);

    template <typename NODE_T>
    static int sendData(NODE_T n) {
      char *buffer = nullptr;
      unsigned int size = DataSerializer::serialize(n, buffer);

      int n_packets = (1 + ((size - 1) / 8)); // get number of packets to store message

      // Start header packet
      if (!CAN.beginExtendedPacket(n.device.__id)) return -1;
      CAN.write(n_packets);
      CAN.write(size);
      if (!CAN.endPacket(1000)) return -1;

      int index = 0; // buffer index
      for (int i = 0; i < n_packets; i++) { // write packet

        if (CAN.beginExtendedPacket(n.device.__id)) return -1;
        for (unsigned int j = 0; j < 8; j++) { // write bytes to packet
          CAN.write(buffer[index]);
          index++;
        }
        if (CAN.endPacket(1000)) return -1;
      }
      return 0;
    }

    template <typename NODE_T>
    static NODE_T getData(NODE_T default_node) {
      default_node.device.__emptyData = true;
      CAN.filterExtended(_id);

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
          return default_node;
        }
      }

      char buffer[message_size];
      int counter = 0;
      for (int i = 0; i < message_n_packets; i++) {
        packetSize = CAN.parsePacket(); // between 1-8 bytes
        while (CAN.available()) {
          int buf_byte = CAN.read();
          if (buf_byte < 0) return default_node;
          buffer[counter] = buf_byte;
          counter++;
        }
      }

      return DataSerializer::deserialize<NODE_T>(buffer, message_size);
    }

   private:
    static long _baudRate;
    static unsigned int _id;
  };
}

#endif