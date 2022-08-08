#ifdef ARDUINO
#include <Arduino.h>
#include "comms.h"

static Comms<CommsSerial> comms;

void setup() {
  comms.start();
}

void loop() {
  
}
#endif