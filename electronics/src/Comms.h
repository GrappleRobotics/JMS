#ifndef COMMS_H
#define COMMS_H

#define k1Mbs 1000E3
#define k500Kbs 500E3
#define k250Kbs 250E3
#define k200Kbs 200E3
#define k125Kbs 125E3
#define k100Kbs 100E3
#define k80Kbs 80E3
#define k50Kbs 50E3
#define k40Kbs 40E3
#define K20Kbs 20E3
#define k10Kbs 10E3
#define k5Kbs 5E3

namespace Comms {
  enum class Type {
    kMaster = 0x0,
    kRedDS = 0x1,
    kBlueDs = 0x2,
    kOther = 0x3
  };


  class Comm {
   public:
    /**
     * @brief Construct a new Comm bus (uses id param if type is kOther)
     * 
     * @param type 
     * @param id 
     */
    Comm(Type type, int id = (int)Type::kOther) {
      _type = type;
      if (type == Type::kOther) {
        _id = id;
      } else {
        _id = (int)type;
      }
    }

   private:
    Type _type;
    int _id;
  };
}

#endif