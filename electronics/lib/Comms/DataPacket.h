#ifndef DATAPACKET_H
#define DATAPACKET_H

#include <MsgPack.h>

namespace Comms {
namespace Message {
  namespace Common {
    /**
     * @brief Common light properties with modes
     * 
     */
    struct Lights {
      enum class Mode {
        kOff        = 0,
        kConstant   = 1,
        kPulse      = 2,
        kChase      = 3,
        kRainbow    = 4
      };

      unsigned int mode; // {off = 0, constant = 1, pulse = 2, chase = 3, rainbow = 4}
      unsigned int speed; // in ms

      struct BRGB {
        byte brightness;
        byte r;
        byte g;
        byte b;
        MSGPACK_DEFINE(brightness, r, g, b);
      };

      BRGB leds;

      void setLights(Mode m, BRGB brgb) {
        mode = (int)m;
        leds = brgb;
      }

      Mode getMode() {
        return (Mode)mode;
      }

      MSGPACK_DEFINE(mode, speed, leds);
    };

    /**
     * @brief Device, provides id and if the packet is empty
     */
    struct Device {
      int __id;
      bool __emptyData = false;
      enum class Type {
        kMaster     = 0x0,
        kRedDS      = 0x1,
        kBlueDS     = 0x2,
        kOther      = 0x3
      };

      void setType(Type t, int id = (int)Type::kOther) { 
        __id = t == Type::kOther ? id : (int)t;
      }

      Type getType() {
        return (Type)__id;
      }

      MSGPACK_DEFINE(__id, __emptyData);
    };
  }

  namespace Nodes {
    /**
     * @brief Alliance node, Red/Blue
     * Sent from any node to alliance node
     */
    struct Alliance {
      Common::Device device; // id also acts as alliance/node type
      bool field_estop;
      bool estop1;
      bool estop2;
      bool estop3;
      Common::Lights lights;

      MSGPACK_DEFINE(device, estop1, estop2, estop3, lights);
    };

    /**
     * @brief Scoring table node
     * Sent from any node to scoring table node
     * 
     */
    struct ScoringTable {
      Common::Device device; // id also acts as alliance/node type
      bool estop;
      Common::Lights lights;

      MSGPACK_DEFINE(device, estop, lights);
    };
  }
} // message
} // comms

#endif