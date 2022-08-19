#include <gtest/gtest.h>

#include "Comms/messages.h"

template<class Vec1, class Vec2>
void expect_vec_eq(const Vec1 &a, const Vec2 &b) {
  EXPECT_EQ(a.size(), b.size());
  for (auto i = std::begin(a), j = std::begin(b); i != std::end(a); i++, j++) {
    EXPECT_EQ(*i, *j);
  }
}

TEST(MessagesTest, Role) {
  std::vector<uint8_t> vec;
  auto inserter = std::back_inserter(vec);

  pack(Role::JMS, inserter);
  pack(Role::RedDs, inserter);
  pack(Role::BlueDs, inserter);

  expect_vec_eq(vec, std::vector<uint8_t>{ 0, 3, 2 });

  auto it = std::begin(vec);

  EXPECT_EQ(unpack<Role>(it), Role::JMS);
  EXPECT_EQ(unpack<Role>(std::vector<uint8_t>::iterator(it)), Role::RedDs); // Ensure we can split the iterator
  EXPECT_EQ(unpack<Role>(it), Role::RedDs);
  EXPECT_EQ(unpack<Role>(it), Role::BlueDs);
}

TEST(MessagesTest, EstopStates) {
  EstopStates estops{ true, { true, false, true }, { false, false, true } };
  uint8_t b;
  pack(estops, &b);
  EXPECT_EQ(b, 0b01001011);
}

TEST(MessagesTest, Colour) {
  uint8_t data[] { 0xCA, 0xBE, 0xEF };
  Colour colour{ 0xCA, 0xBE, 0xEF };
  EXPECT_EQ(unpack<Colour>(std::begin(data)), colour );
}

TEST(MessagesTest, PackUnpack) {
  std::vector<uint8_t> buf;

  // Ping
  {
    AddressedMessage orig{ Role::RedDs, MessagePing{} };
    pack(orig, std::back_inserter(buf));
    expect_vec_eq(buf, std::vector<uint8_t>{ 0x03, 0x00 });
    
    auto unpacked = unpack<AddressedMessage>(std::begin(buf));
    EXPECT_EQ(unpacked.role, orig.role);
    EXPECT_NO_THROW(std::get<MessagePing>(unpacked.msg));
  }

  buf.clear();

  // Estops (Pack)
  {
    AddressedMessage orig{ Role::ScoringTable, MessageEstops { { true, { false, true, false }, { true, true, false } } } };
    pack(orig, std::back_inserter(buf));
    expect_vec_eq(buf, std::vector<uint8_t>{ 0x01, 0x01, 0b00110101 });
  }

  buf.clear();

  // Lights (Unpack)
  {
    uint8_t unpack_buf[] { 0x02, 0x02, 0x02, /* Lights 1 */ 0x1, 0xAB, 0xCD, 0xEF, /* Lights 2 */ 0x3, 0x12, 0x34, 0x56, 0x98, 0x05 };
    auto it = std::begin(unpack_buf);
    auto unpacked = unpack<AddressedMessage>(it);

    // Should read fully
    EXPECT_EQ(it, std::end(unpack_buf));

    EXPECT_EQ(unpacked.role, Role::BlueDs);
    auto lights = std::get<MessageSetLights>(unpacked.msg);
    EXPECT_EQ(lights.lights.size(), 2);
    // Lights 1 - Constant AB CD EF
    auto l1 = std::get<LightModeConstant>(lights.lights[0]);
    Colour colour1{ 0xAB, 0xCD, 0xEF };
    EXPECT_EQ(l1.colour, colour1);
    // Lights 2 - Chase 12 34 56 for 1432ms
    auto l2 = std::get<LightModeChase>(lights.lights[1]);
    Colour colour2{ 0x12, 0x34, 0x56 };
    EXPECT_EQ(l2.colour, colour2);
    EXPECT_EQ(l2.duration, 1432);
  }
}