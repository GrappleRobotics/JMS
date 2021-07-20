#include <mbed.h> // bruh, never include this twice... everything just dies

#include <iostream>
#include "Handles.h"
#include "Config.h"


#ifdef MODE

/**
 * RAM/BAM Controllers
 */
#if defined(RAM) || defined(BAM)
#include "Elements/PowerPort/PowerPort.h"
HandleController(PowerPort)
#endif

#if defined(SGM)
HandleController(ShieldGen)
#endif

#if defined(STM)
HandleController(ScoringTable)
#endif

#else
#error MODE NOT DEFINED [RAM, BAM, STM, SGM]
#endif