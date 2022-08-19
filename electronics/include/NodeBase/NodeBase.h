#pragma once

class NodeBase {
 public:
  NodeBase(long serial_br, long can_br) {};
  ~NodeBase() {};

  virtual void init() {};
  virtual void onUpdate() {};

 protected:
  long _serial_br;
  long _can_br;
};