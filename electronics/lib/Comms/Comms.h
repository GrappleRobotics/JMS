#ifndef COMMS_H
#define COMMS_H

#include "SerialComms/SerialComms.h"
#include "CANComms/CANComms.h"

namespace Comms {

  class Comm {
   public:
    static void init(int id) {
      CANComm::setID(id);
    }

    static void init(int id, long serial_br) {
      init(id);
      SerialComm::setBaudRate(serial_br);
    }

    static void init(int id, long serial_br, long can_br) {
      init(id, serial_br);
      CANComm::setBaudRate(can_br);
    }

    static void start() {
      CANComm::start();
      SerialComm::start();
    }

    static void stop() {
      CANComm::stop();
      SerialComm::stop();
    }

    static void setBaudRate(int serial_br, int can_br) {
      SerialComm::setBaudRate(serial_br);
      CANComm::setBaudRate(can_br);
    }

    template <typename NODE_T>
    static int sendData(NODE_T n) {
      if (n.device.__id > 0x0) { // if not jms (using CAN bus)
        return CANComm::sendData(n);
      } else {
        return SerialComm::sendData(n);
      }
    }
    
    template <typename NODE_T>
    static NODE_T getData(NODE_T default_node, bool force_serial = false) { // populate default node with JMS id if data is over serial
      if (force_serial || default_node.device.__id == 0x0) {
        return SerialComm::getData(default_node);
      } else {
        return CANComm::getData(default_node);
      }
    }
  };
}

#endif