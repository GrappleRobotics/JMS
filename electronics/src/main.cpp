#include <mbed.h>
#include <iostream>
#include "Network.h"
#include "OLED.h"
#include <stdio.h>
#include <string>

// @TODO Sensor Beam breaks/Reflections
int main() {
	// OLED oled(D14, D15);
	DigitalIn userButton(USER_BUTTON);
	AnalogIn opticalIn(A0);
	

	int goal = 0;

	// If user pressed blue button. End program
	while (userButton != 1) {
		// std::cout << "Sensor Input: " << opticalIn.read() << std::endl;
		// std::cout << "Sensor Input Short: " << opticalIn.read_u16() << std::endl;
		std::cout << "Sensor Input pls: " << opticalIn.read_u16() << std::endl;

		// oled.get()->writeString(0,0, "Input: " + to_string(opticalIn.read()));
	}
	
	// oled.~OLED();
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