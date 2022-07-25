#include <Arduino.h>

const int LED_PIN = 13;
const int INTERRUPT_PIN = 2;

volatile bool ledState = LOW;

void myISR() {
  ledState = !ledState;
}

void setup() {
  pinMode(LED_PIN, OUTPUT);
  pinMode(INTERRUPT_PIN, INPUT_PULLUP);
  attachInterrupt(digitalPinToInterrupt(INTERRUPT_PIN), myISR, FALLING);
}

void loop() {
  digitalWrite(LED_PIN, ledState);
}