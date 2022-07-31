#ifndef NODEBASE_H
#define NODEBASE_H

#include "Comms.h"

class NodeBase {
 public:
  NodeBase(unsigned int id);
  ~NodeBase();
  virtual void init() {};
  virtual void loop() {};

 protected:
  unsigned int _id;
};

#endif