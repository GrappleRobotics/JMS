#pragma once

class NodeBase {
 public:
  NodeBase(unsigned long serial_br, unsigned long can_br) : _serial_br(serial_br), _can_br(can_br) {};
  ~NodeBase() {};

  virtual void init() {};
  virtual void onUpdate() {};

 protected:
  unsigned long _serial_br;
  unsigned long _can_br;
};