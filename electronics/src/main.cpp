#include <Arduino.h>

#include "Comms/Comms.h"
#include "LEDStrips.h"
#include "InterruptButton.h"

// Uncomment the device type
#define MASTER_D Comms::Message::Common::Device::Type::kMaster // can you master the D?
// #define RED_ALLIANCE_D Comms::Message::Common::Device::Type::kRedDS
// #define BLUE_ALLIANCE_D Comms::Message::Common::Device::Type::kBlueDs

#if defined(MASTER_D)
  #define DEVICE MASTER_D
  #define NODE_MESSAGE Comms::Message::Nodes::ScoringTable
#elif defined(RED_ALLIANCE_D)
  #define DEVICE RED_ALLIANCE_D
  #define NODE_MESSAGE Comms::Message::Nodes::Alliance
#elif defined(BLUE_ALLIANCE_D)
  #define DEVICE BLUE_ALLIANCE_D
  #define NODE_MESSAGE Comms::Message::Nodes::Alliance
#endif

// volatile bool state = false;
void onInterrupt() {
  // @TODO: Something actually useful...
  // state = !state;
  // digitalWrite(LED_BUILTIN, HIGH);
  // state = true;
}

InterruptButton e_r1(4, &onInterrupt);
InterruptButton e_r2(5, &onInterrupt);
InterruptButton e_r3(6, &onInterrupt);

InterruptButton e_b1(7, &onInterrupt);
InterruptButton e_b2(8, &onInterrupt);
InterruptButton e_b3(9, &onInterrupt);

InterruptButton e_mst(A0, &onInterrupt);

LED::Strip strip;

Comms::Message::Nodes::Alliance a;
Comms::Message::Nodes::ScoringTable s;

void setup() {
  Serial.begin(9600);
  strip.create<WS2812<2, BRG>>(120);

  // Setup comms
  Comms::Comm::setBaudRate(k500Kbs);
  Comms::Comm::setNodeID(DEVICE); // set our if to the device type (we only listen for data being sent to us)
  Comms::Comm::start(); // start the Comm service

  // Default state for nodes
  #if defined(MASTER_D)
    a.lights.setLights(Comms::Message::Common::Lights::Mode::kConstant, {0,255,0});
  #elif defined(RED_ALLIANCE_D)
    a.lights.setLights(Comms::Message::Common::Lights::Mode::kConstant, {255,0,0});
  #elif defined(BLUE_ALLIANCE_D)
    a.lights.setLights(Comms::Message::Common::Lights::Mode::kConstant, {0,0,255});
  #endif
}

void loop() {
  #ifdef MASTER_D
    s = Comms::Comm::getData(s); // don't really need it. Because nothing should be sending to the scoring table. But eh
    if (e_mst.isTriggered()) {
      a.field_estop = true;
    }
  #elif defined(RED_ALLIANCE_D) || defined(BLUE_ALLIANCE_D)
    
  #endif
}