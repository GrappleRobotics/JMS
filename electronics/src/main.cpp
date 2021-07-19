#include "config.h"

#ifdef RAM
#define MODE 1
#endif

#ifdef BAM
#define MODE 0
#endif

#ifdef SGM
#define MODE 2
#endif

#ifdef STM
#define MODE 3
#endif


/**
 * System headers
 */
#include <mbed.h>



#ifdef MODE
/**
 * Element headers
 */
#include 

int main() {
	return 0;
}
#else
#error MODE NOT DEFINED [RAM, BAM, STM, SGM]
#endif