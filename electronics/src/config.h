// #ifndef CONFIG_H
// #define CONFIG_H

// #include <mbed.h>
// #include <iostream>
// #include <string>
// /**
//  *
//  * -----------------------------------------------
//  * SET MODE [Type of controller to be deployed to]
//  * -----------------------------------------------
//  * 
//  */

// #define RAM // Red Alliance controller mode
// // #define BAM // Blue Alliance controller mode
// // #define SGM // Shield Generator controller mode
// // #define STM // Scoring Table controller mode

// #ifdef RAM
// #define MODE 0
// #endif

// #ifdef BAM
// #define MODE 1
// #endif

// #ifdef SGM
// #define MODE 2
// #endif

// #ifdef STM
// #define MODE 3
// #endif

// #define global static constexpr // Constant experssion accessable to all (cannot be modified)

// /**
//  * Config for Power port
//  */

// struct Config {


// 	#ifdef RAM
// 	#define IP "10.0.100.4"
// 	global int DISPLAY_SDA_PORT = 14;
// 	global int DISPLAY_SCL_PORT = 15;
// 	#endif

// 	#ifdef BAM
// 	#define IP "10.0.100.5"
// 	global int DISPLAY_SDA_PORT = 14;
// 	global int DISPLAY_SCL_PORT = 15;
// 	#endif


// 	/**
// 	 * Shield generator config
// 	 */
// 	#ifdef SGM
// 	#define IP "10.0.100.3"
// 	global int DISPLAY_SDA_PORT = 14;
// 	global int DISPLAY_SCL_PORT = 15;
// 	#endif

// 	/**
// 	 * Scoring table config
// 	 */
// 	#ifdef STM
// 	#define IP "10.0.100.2"
// 	global int DISPLAY_SDA_PORT = 14;
// 	global int DISPLAY_SCL_PORT = 15;
// 	#endif

// };

// #endif // CONFIG_H