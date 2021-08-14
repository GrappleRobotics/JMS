#ifndef LED_STRIP_H
#define LED_STRIP_H

#include <vector>
#include <neopixel.h>

namespace CJ_NEO {
	struct RGB {
		uint8_t red, green, blue;
	};

	struct LED {
		neopixel::Pixel pixel;

		/**
		 * Set, RGB 0-255
		 */
		void set(RGB rgb) {
			pixel.red = rgb.red;
			pixel.green = rgb.green;
			pixel.blue = rgb.blue;
		};
	};

	/**
	 * LED Strip
	 * 
	 * Create an LED strip using an analog input pin & the number of LED's
	 */
	class LED_Strip {
	 public:
		LED_Strip(PinName pin, int count) {
			_pixels.resize(count);
			_pixelArray = new neopixel::PixelArray(pin);
		}

		~LED_Strip() {
			_pixelArray = nullptr;
			delete _pixelArray;
		}

		/**
		 * Set entire strip statically
		 */
		void setStrip(RGB rgb) {
			for (size_t i = 0; i < _pixels.size(); i++) {
				set(i, rgb);
			}
		}

		/**
		 * Set individual leds in strip
		 */
		void set(int pos, RGB rgb) {
			_pixels[pos].set(rgb);
		}

		/**
		 * Update strip 
		 * (Called when setting regardless)
		 */
		void update() {
			neopixel::Pixel buffer[_pixels.size()];
			for (size_t i = 0; i < _pixels.size(); i++) {
				buffer[i] = _pixels[i].pixel;
			}
			_pixelArray->update(buffer, _pixels.size());
		}

	 private:
		std::vector<LED> _pixels;
		neopixel::PixelArray *_pixelArray;
	};
}


#endif