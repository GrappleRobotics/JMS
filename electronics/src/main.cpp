#include <Arduino.h>

#define SERIAL_BAUD 115200
#define CAN_BAUD 500E3

#define NODE_TYPE 0 // 0 = master, 1 = red, 2 = blue

#if NODE_TYPE==0
#include "ScoringTable.h"
#define NODE_INSTANTIATE ScoringTable __node(SERIAL_BAUD, CAN_BAUD)
#elif NODE_TYPE==1
#include "Alliance.h"
#define NODE_INSTANTIATE Alliance __node(Comms::Message::Common::Device::Type::kRedDS, SERIAL_BAUD, CAN_BAUD)
#elif NODE_TYPE==2
#include "Alliance.h"
#define NODE_INSTANTIATE Alliance __node(Comms::Message::Common::Device::Type::kBlueDS, SERIAL_BAUD, CAN_BAUD)
#else
#include "ScoringTable.h"
#define NODE_INSTANTIATE ScoringTable __node(SERIAL_BAUD, CAN_BAUD)
#endif

#define NODE_MAIN(node) void setup() { Serial.begin(9600); Serial.println("Node Start"); node.init(); } void loop() { node.loop(); }

NODE_INSTANTIATE;
NODE_MAIN(__node)