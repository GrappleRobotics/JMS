#include "config.h"

#if NODE_TYPE==0
#include "ScoringTable/ScoringTable.h"
#define NODE_INSTANTIATE ScoringTable __node(SERIAL_BAUD, CAN_BAUD)
#elif NODE_TYPE==1
// red alliance
#elif NODE_TYPE==2
// blue alliance
#elif NODE_TYPE==3
#include "ScoringTable/ScoringTableSlave.h"
#define NODE_INSTANTIATE ScoringTableSlave __node;
#else

#endif

#ifdef ARDUINO
#include <Arduino.h>
#define NODE_MAIN(node) void setup() { node.init(); } void loop() { node.onUpdate(); }
#else
#define NODE_MAIN(node) int main() { node.init(); while(true){node.onUpdate();} }
#endif

NODE_INSTANTIATE;
NODE_MAIN(__node);