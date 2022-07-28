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

    static void setNodeID(Message::Common::ID::Type t, int id = (int)Message::Common::ID::Type::kOther) {
      _id.setType(t, id);
    }

    template <typename NODE_T>
    static void sendDataTo(NODE_T n, int id) {
      // _packer.clear();
      // _packer.serialize(n);
      // CAN.begin
    }

    template <typename NODE_T>
    static NODE_T getDataFrom(int id) {

    }

   private:
    static MsgPack::Packer _packer;
    static MsgPack::Unpacker _unpacker;
    static long _baudRate;
    static Message::Common::ID _id;
  };
}

#endif