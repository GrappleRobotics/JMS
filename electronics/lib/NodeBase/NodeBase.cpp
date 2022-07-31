#include "NodeBase.h"

using namespace Comms;

NodeBase::NodeBase(unsigned int id) : _id(id) {
  Comm::init(id);
}

NodeBase::~NodeBase() {}