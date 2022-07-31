#ifndef DATA_SERIALIZER_H
#define DATA_SERIALIZER_H

#include "MsgPack.h"

namespace Comms {
  class DataSerializer {
   public:
    
    template <typename DATA>
    static int serialize(DATA data, char *buffer) {
      MsgPack::Packer packer;
      packer.serialize(data);
      buffer = (char *)packer.data();
      return packer.size();
    }

    template <typename DATA>
    static DATA deserialize(char *buffer, unsigned int size) {
      MsgPack::Unpacker unpacker;
      unpacker.feed((uint8_t *)buffer, size);
      DATA d;
      unpacker.deserialize(d);
      return d;
    }
  };
}

#endif