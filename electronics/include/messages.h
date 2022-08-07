#include <cstdint>
#include <variant>
#include <vector>
#include <bitset>

template<class... Ts> struct overloaded : Ts... { using Ts::operator()...; };
template<class... Ts> overloaded(Ts...) -> overloaded<Ts...>;

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

struct LightModeOff {};
struct LightModeConstant { Colour colour; };
struct LightModePulse { Colour colour; uint16_t duration; };
struct LightModeChase { Colour colour; uint16_t duration; };
struct LightModeRainbow { uint16_t duration; };
using LightMode = std::variant<LightModeOff, LightModeConstant, LightModePulse, LightModeChase, LightModeRainbow>;

template<class It>
void _unpack(It &&it, LightMode &mode) {
  uint8_t variant = *it++;
  switch (variant) {
  case 0:
    mode = LightModeOff{};
    break;
  case 1:
    mode = LightModeConstant{ unpack<Colour>(it) };
    break;
  case 2:
    mode = LightModePulse{ unpack<Colour>(it), unpack<uint16_t>(it) };
    break;
  case 3:
    mode = LightModeChase{ unpack<Colour>(it), unpack<uint16_t>(it) };
    break;
  case 4:
    mode = LightModeRainbow{ unpack<uint16_t>(it) };
    break;
  default:
    break;
  }
};

struct MessagePing {};
struct MessageEstops { EstopStates estops; };
struct MessageSetLights { std::vector<LightMode> lights; };
using Message = std::variant<MessagePing, MessageEstops, MessageSetLights>;

template<class It>
void _unpack(It &&it, Message &msg) {
  uint8_t variant = *it++;
  switch (variant) {
  case 0: // Ping
    msg = MessagePing{};
    break;
  case 2: { // Set Lights
    uint8_t n = *it++;
    std::vector<LightMode> lights;
    for (auto i = 0; i < n; i++)
      lights.push_back(unpack<LightMode>(it));
    msg = MessageSetLights{ lights };
    break;
  }
  default:
    break;
  }
}

template<class OutputIterator>
void pack(const Message &msg, OutputIterator it) {
  std::visit(overloaded {
    [](auto any) {},
    [&it](const MessagePing &ping) { *it++ = 0; },
    [&it](const MessageEstops &estops) { *it++ = 1; pack(estops.estops, it); }
  }, msg);
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