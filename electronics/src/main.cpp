#define SERIAL_BAUD 115200
#define CAN_BAUD 500E3

#define NODE_TYPE 0 // 0 = master, 1 = red, 2 = blue

#if NODE_TYPE==0
#include "ScoringTable/ScoringTable.h"
#define NODE_INSTANTIATE ScoringTable __node(SERIAL_BAUD, CAN_BAUD)
#elif NODE_TYPE==1
// red alliance
#elif NODE_TYPE==2
// blue alliance
#else

#endif

#ifdef ARDUINO
#include <Arduino.h>
#define NODE_MAIN(node) void setup() { node.init(); } void loop() { node.onUpdate(); }
#endif

NODE_INSTANTIATE;
NODE_MAIN(__node);

// #ifdef ARDUINO
// #include <Arduino.h>
// #include "comms.h"

// static Comms<CommsSerial> comms;

// void setup() {
//   comms.start();

//   pinMode(A2, INPUT_PULLUP);
//   pinMode(4, INPUT_PULLUP);
//   pinMode(5, INPUT_PULLUP);
//   pinMode(6, INPUT_PULLUP);
//   pinMode(7, INPUT_PULLUP);
//   pinMode(8, INPUT_PULLUP);
//   pinMode(9, INPUT_PULLUP);
// }

// void loop() {
//   // auto msg = comms.poll();
//   // if (msg.has_value()) {

//   // }

//   const MessageEstops estops{
//     EstopStates {
//       !digitalRead(A2), { !digitalRead(4), !digitalRead(5), !digitalRead(6) }, { !digitalRead(7), !digitalRead(8), !digitalRead(9) }
//     }
//   };

//   AddressedMessage new_msg{
//     Role::ScoringTable,
//     estops
//   };
//   comms.write(new_msg);
//   delay(100);
// }
// #endif