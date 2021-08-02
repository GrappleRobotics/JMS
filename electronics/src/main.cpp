#include <mbed.h> // bruh, never include this twice... everything just dies
#include <rtos.h>

#include <iostream>
#include "Handles.h"
#include "Config.h"

#ifdef MODE

/**
 * RAC/BAC Controllers
 */
#if defined(RAC) || defined(BAC)
#include "Controllers/RAC_BAC/RAC_BAC.h"
#include "Elements/PowerPort/PowerPort.h"
HandleController(RAC_BAC_Controller)
#endif

/**
 * SGC Controllers
 */
#if defined(SGC)
HandleController(SGC_Controller)
#endif

/**
 * STC Controllers
 */
#if defined(STC)
#include "Controllers/STC/STC.h"
HandleController(STC_Controller)
#endif

/**
 * Auto configure mode from network
 */
#if defined(AUTO_CONFIG)
// @TODO
#endif


#else
#error MODE NOT DEFINED [RAC, BAC, STC, SGC, AUTO_CONFIG]
#endif