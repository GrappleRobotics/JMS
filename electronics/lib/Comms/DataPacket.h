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
      Device(int id = 0, bool empty = false) : __id(id), __emptyData(empty) {}
      int __id;
      bool __emptyData = false;
      enum class Type {
        kJMS        = 0x0,
        kMaster     = 0x1,
        kRedDS      = 0x2,
        kBlueDS     = 0x3,
        kOther      = 0x4
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
      Common::Device device{(int)Common::Device::Type::kMaster, false}; // id also acts as alliance/node type
      bool estop;
      Common::Lights lights;

      MSGPACK_DEFINE(device, estop, lights);
    };
    
    /**
     * @brief JMS Node/Server
     * Sent from any node (capable) to the JMS
     */
    struct JMS {
      Common::Device device{(int)Common::Device::Type::kJMS, false};
      bool estop;

      bool r1_estop;
      bool r2_estop;
      bool r3_estop;

      bool b1_estop;
      bool b2_estop;
      bool b3_estop;

      MSGPACK_DEFINE(device, estop, r1_estop, r2_estop, r3_estop, b1_estop, b2_estop, b3_estop);
    };
  }
} // message
} // comms

#endif