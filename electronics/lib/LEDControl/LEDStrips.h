#ifndef LED_STRIPS_H
#define LED_STRIPS_H

#include <FastLED.h>
#include <Vector.h>

namespace LED {

  class Strip {
   public:
    ~Strip() {
      free(_strip);
    }

    template <typename CHIPSET>
    void create(int size) {
      static CHIPSET c;
      _size = size;
      _strip = (CRGB*)calloc(size, sizeof(CRGB));
      FastLED.addLeds(&c, _strip, size).setCorrection(TypicalLEDStrip);
      FastLED.clear();
      FastLED.show();
    }

    void setBrightness(byte value);

    void set(unsigned int index, CRGB rgb, bool noShow = false);
    void set(CRGB rgb, bool noShow = false);

    // Animations


    void setRainbow(int speed, bool noShow = false);
    void setWave(CRGB colour, int waveSize, int speed, bool noShow = false);
    void setPulse(CRGB colour, int speed, bool noShow = false);

    size_t getSize() {
      return _size;
    }

   private:
    CRGB *_strip = nullptr;
    size_t _size;
  };
}

#endif