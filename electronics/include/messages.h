#include <stdint.h>

#include "lib/tagged_variant.h"

template<class It>
void _unpack(It &&it, uint16_t &u16) {
  u16 = (*it++) | (*it++ << 8);
}

template<typename T, class It>
T unpack(It &&it) {
  T a;
  _unpack(it, a);
  return a;
}

enum class Role {
  JMS = 0,
  ScoringTable = 1,
  BlueDs = 2,
  RedDs = 3
};

template<class OutputIterator>
void pack(const Role &role, OutputIterator iter) {
  *iter++ = static_cast<uint8_t>(role);
}

template<class It>
void _unpack(It &&it, Role &role) {
  role = static_cast<Role>(*it++);
}

struct EstopStates {
  bool field;
  bool red[3];
  bool blue[3];
};

template<class OutputIterator>
void pack(const EstopStates &states, OutputIterator iter) {
  *iter++ = states.field | (states.red[0] << 1) | (states.red[1] << 2) | (states.red[2] << 3)
                         | (states.blue[0] << 4) | (states.blue[1] << 5) | (states.blue[2] << 6);
}

struct Colour {
  uint8_t red;
  uint8_t green;
  uint8_t blue;
};

bool operator==(const Colour &lhs, const Colour &rhs) {
  return lhs.red == rhs.red && lhs.green == rhs.green && lhs.blue == rhs.blue;
}

template<class It>
void _unpack(It &&it, Colour &c) {
  c = Colour { *it++, *it++, *it++ };
}

struct LightModeOff { static const size_t tag = 0; };
struct LightModeConstant { static const size_t tag = 1; Colour colour; };
struct LightModePulse { static const size_t tag = 2; Colour colour; uint16_t duration; };
struct LightModeChase { static const size_t tag = 3; Colour colour; uint16_t duration; };
struct LightModeRainbow { static const size_t tag = 4; uint16_t duration; };
using LightMode = tagged_variant<LightModeOff, LightModeConstant, LightModePulse, LightModeChase, LightModeRainbow>;

template<class It>
void _unpack(It &&it, LightMode &mode) {
  uint8_t variant = *it++;
  switch (variant) {
  case LightModeOff::tag:
    mode = LightModeOff{};
    break;
  case LightModeConstant::tag:
    mode = LightModeConstant{ unpack<Colour>(it) };
    break;
  case LightModePulse::tag:
    mode = LightModePulse{ unpack<Colour>(it), unpack<uint16_t>(it) };
    break;
  case LightModeChase::tag:
    mode = LightModeChase{ unpack<Colour>(it), unpack<uint16_t>(it) };
    break;
  case LightModeRainbow::tag:
    mode = LightModeRainbow{ unpack<uint16_t>(it) };
    break;
  default:
    break;
  }
};

struct MessagePing { static const size_t tag = 0; };
struct MessageEstops { static const size_t tag = 1; EstopStates estops; };
struct MessageSetLights { static const size_t tag = 2; LightMode lights[4]; };
using Message = tagged_variant<MessagePing, MessageEstops, MessageSetLights>;

template<class It>
void _unpack(It &&it, Message &msg) {
  uint8_t variant = *it++;
  switch (variant) {
  case MessagePing::tag: // Ping
    msg = MessagePing{};
    break;
  case MessageSetLights::tag: { // Set Lights
    uint8_t n = *it++;
    MessageSetLights msl;

    size_t i;
    for (i = 0; i < n; i++)
      msl.lights[i] = unpack<LightMode>(it);
    for (; i < 4; i++)
      msl.lights[i] = LightModeOff{};

    msg = msl;
    break;
  }
  default:
    break;
  }
}

template<class OutputIterator>
void pack(const Message &msg, OutputIterator it) {
  *it++ = msg.tag();
  if (msg.is<MessageEstops>()) {
    pack(msg.get<MessageEstops>(), it);
  }
  // std::visit(overloaded {
  //   [](auto any) {},
  //   [&it](const MessagePing &ping) { *it++ = 0; },
  //   [&it](const MessageEstops &estops) { *it++ = 1; pack(estops.estops, it); }
  // }, msg);
}

struct AddressedMessage {
  Role role;
  Message msg;
};

template<class It>
void _unpack(It &&it, AddressedMessage &msg) {
  msg.role = unpack<Role>(it);
  msg.msg = unpack<Message>(it);
}

template<class OutputIterator>
void pack(const AddressedMessage &msg, OutputIterator it) {
  pack(msg.role, it);
  pack(msg.msg, it);
}