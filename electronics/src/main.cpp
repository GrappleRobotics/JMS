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
HandleController(RAM_BAM_Controller)
#endif

/**
 * SGM Controllers
 */
#if defined(SGM)
HandleController(SGM_Controller)
#endif

/**
 * STM Controllers
 */
#if defined(STM)
#include "Controllers/STM/STM.h"
HandleController(STM_Controller)
#endif

#else
#error MODE NOT DEFINED [RAM, BAM, STM, SGM]
#endif