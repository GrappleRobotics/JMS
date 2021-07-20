#include <mbed.h> // bruh, never include this twice... everything just dies
#include <rtos.h>

#include <iostream>
#include "Handles.h"
#include "Config.h"

#ifdef MODE

/**
 * RAM/BAM Controllers
 */
#if defined(RAM) || defined(BAM)
#include "Controllers/RAM_BAM/RAM_BAM.h"
#include "Elements/PowerPort/PowerPort.h"
HandleController(RAM_BAM)
#endif

#if defined(SGM)
HandleController(SGM)
#endif

#if defined(STM)
HandleController(STM)
#endif

#else
#error MODE NOT DEFINED [RAM, BAM, STM, SGM]
#endif