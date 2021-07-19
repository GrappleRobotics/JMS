#include <mbed.h>
#include <iostream>
#include <stdio.h>
#include <string>

#include "Network.h"
#include "OLED.h"
#include "BeamBreak.h"

// @TODO Sensor Beam breaks/Reflections
int main() {
	// OLED oled(D14, D15);
	BeamBreak beamBreak(A0);

	std::cout << "test" << std::endl;

	DigitalIn userButton(USER_BUTTON);
	// AnalogIn beam(A0);

	int goalCount = 0;

	// If user pressed blue button. End program
	// opticalIn.set_reference_voltage(5.0f);
	while (userButton != 1) {
		beamBreak.updateStart(); // START
		
		if (beamBreak.broke()) {
			goalCount++;
		}

		std::cout << "Goal count: " << goalCount << std::endl;
		// printf("Value: %d\n", beam.read_u16());
		// oled.print(goalCount);

		std::cout << "State: " << (int)beamBreak.getStates().first << std::endl;

		// ThisThread::sleep_for(200ms);

		beamBreak.updateEnd(); // END
	}
	
	// oled.~OLED();
	beamBreak.~BeamBreak();
}

// int main() {
//   // Note: https://www.st.com/resource/en/reference_manual/dm00314099-stm32h742-stm32h743-753-and-stm32h750-value-line-advanced-arm-based-32-bit-mcus-stmicroelectronics.pdf
//   // We use Sector 7 of Bank 2 - 0x081E 0000 + 128K

//   const uint32_t CFG_REGION = 0x081E0000;

//   FlashIAP flash;
//   flash.init();

//   char buf1[50] = {0};
//   char buf2[50] = {0};
//   strcpy(buf2, "Hello World ABCD!");

//   int r = flash.read(buf1, CFG_REGION, 50);
//   printf("Read %d - %s\n", r, buf1);

//   ThisThread::sleep_for(100ms);

//   r = flash.program(buf2, CFG_REGION, 50);
//   printf("Programmed %d - %s\n", r, buf2);

//   ThisThread::sleep_for(100ms);

//   r = flash.read(buf1, CFG_REGION, 50);
//   printf("Read %d - %s\n", r, buf1);

//   flash.deinit();

//   while(1) {
//     ThisThread::sleep_for(1s);
//   }
// }