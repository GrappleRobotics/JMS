// #include <mbed.h>
// #include "SSD1306.h"

// /**
//  * Cheap way of using oled screen.
//  * 
//  * Type: SSD1306, 
//  * Size: 128x32, 
//  * Addr: 0x78
//  * Conn: i2c
//  */
// class OLED {
//  public:

// 	OLED(PinName sda, PinName scl, uint8_t addr = SSD1308_SA0) {
// 		i2c = new I2C(sda, scl);
// 		oled = new SSD1306(i2c, addr);
// 	}

// 	~OLED() {
// 		oled->clearDisplay();
// 		delete oled;
// 		delete i2c;
// 	}

// 	/**
// 	 * Only prints numbers 0 - 9 and '+', '-', '.'
// 	 */
// 	void print(std::string value) {
// 		for (size_t i = 0; i < value.size(); i++) {
// 			this->oled->writeBigChar(50, i*16, value[i]);
// 		}
// 	}

// 	/**
// 	 * Only prints numbers 0 - 9 and '+', '-', '.'
// 	 */
// 	void print(int value) {
// 		std::string strVal = to_string(value);
// 		print(strVal);
// 	}


// 	SSD1306 *get() {
// 		return this->oled;
// 	}

// 	I2C *getI2C() {
// 		return this->i2c;
// 	}

//  private:
// 	SSD1306 *oled;
// 	I2C *i2c;
// };