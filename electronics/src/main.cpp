#include <Arduino.h>

#define NODE_TYPE 0 // 0 = master, 1 = red, 2 = blue

#if NODE_TYPE==0
#include "ScoringTable/ScoringTable.h"
#define NODE_INSTANTIATE ScoringTable __node(k500Kbs)
#elif NODE_TYPE==1
#include "Alliance/Alliance.h"
#define NODE_INSTANTIATE Alliance __node(Comms::Message::Common::Device::Type::kRedDS, k500Kbs)
#elif NODE_TYPE==2
#include "Alliance/Alliance.h"
#define NODE_INSTANTIATE Alliance __node(Comms::Message::Common::Device::Type::kBlueDS, k500Kbs)
#else
#include "ScoringTable/ScoringTable.h"
#define NODE_INSTANTIATE ScoringTable __node(k500Kbs)
#endif

#define NODE_MAIN(node) void setup() { Serial.begin(9600); Serial.println("Node Start"); node.init(); } void loop() { node.loop(); }

NODE_INSTANTIATE;
NODE_MAIN(__node)