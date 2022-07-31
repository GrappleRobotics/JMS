#ifndef SERIAL_COMMS_H
#define SERIAL_COMMS_H

#include <Arduino.h>
#include "Serializer/DataSerializer.h"

namespace Comms {
  class SerialComm : public DataSerializer {
   public:
    static void start();
    static void stop();

    static void setBaudRate(long br);

    template <typename NODE_T>
    static int sendData(NODE_T n) {
      char *buffer = nullptr;
      unsigned int size = serialize(n, buffer);

      if (Serial.available() <= 0) return -1;
      Serial.write(buffer, size);
      return 0;
    }

    template <typename NODE_T>
    static NODE_T getData(NODE_T default_node) {
      if (Serial.available() <= 0) return default_node;

      char *buffer;
      unsigned int size = Serial.readBytes(buffer, 256);

      return deserialize<NODE_T>(buffer, size);
    }

   private:
    static long _baudRate;
  };
}

#endif