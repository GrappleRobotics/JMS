#ifndef CONFIG_H
#define CONFIG_H

#include "Handles.h"
#include <PinNames.h>

/**
 *
 * -----------------------------------------------
 * SET MODE [Type of controller to be deployed to]
 * -----------------------------------------------
 * 
 */

// #define AUTO_CONFIG // Auto configure depending on JMS (runtime flash)
// #define RAC // Red Alliance controller mode
// #define BAC // Blue Alliance controller mode
// #define SGC // Shield Generator controller mode
#define STC // Scoring Table controller mode

#ifdef RAC
#define MODE 0
#endif

#ifdef BAC
#define MODE 1
#endif

#ifdef SGC
#define MODE 2
#endif

#ifdef STC
#define MODE 3
#endif

#ifdef AUTO_CONFIG
#define MODE 4
#endif

/**
 * Config for Power port
 */
#ifdef RAC
static const int DISPLAY_SDA_PORT = 14;
static const int DISPLAY_SCL_PORT = 15;

static const PinName INNER_BB_PORT = A0;
static const PinName OUTER_BB_PORT = A1;
static const PinName LOWER_BB_PORT = A2;
static const PinName OUTER_LED_PORT = D7;

static const PinName E_STOP1_1 = D16;
static const PinName E_STOP1_2 = D17;

static const PinName E_STOP2_1 = D18;
static const PinName E_STOP2_2 = D19;

static const PinName E_STOP3_1 = D20;
static const PinName E_STOP3_2 = D21;
#endif

#ifdef BAC
static const int DISPLAY_SDA_PORT = 14;
static const int DISPLAY_SCL_PORT = 15;

static const PinName INNER_BB_PORT = A0;
static const PinName OUTER_BB_PORT = A1;
static const PinName LOWER_BB_PORT = A2;
static const PinName OUTER_LED_PORT = D7;

static const PinName E_STOP1_1 = D16;
static const PinName E_STOP1_2 = D17;

static const PinName E_STOP2_1 = D18;
static const PinName E_STOP2_2 = D19;

static const PinName E_STOP3_1 = D20;
static const PinName E_STOP3_2 = D21;
#endif


/**
 * Shield generator config
 */
#ifdef SGC
static const int DISPLAY_SDA_PORT = 14;
static const int DISPLAY_SCL_PORT = 15;
#endif

/**
 * Scoring table config
 */
#ifdef STC
static const int DISPLAY_SDA_PORT = 14;
static const int DISPLAY_SCL_PORT = 15;

static const PinName ABORT_1 = D16;
static const PinName ABORT_2 = D17;
#endif

#endif // CONFIG_H