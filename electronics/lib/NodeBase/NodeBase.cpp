#include "NodeBase.h"

NodeBase::NodeBase(Comms::Message::Common::Device::Type t, int id, long baudRate) {
  _type = t;

  Comms::Comm::setBaudRate(baudRate);
  Comms::Comm::setNodeID(t, id);
  Comms::Comm::start();
}

NodeBase::~NodeBase() {
  Comms::Comm::stop();
}