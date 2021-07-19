/**
 * 
 * -----------------------------------------------
 * SET MODE [Type of controller to be deployed to]
 * -----------------------------------------------
 * 
 */

#define RAM // Red Alliance controller mode
// #define BAM // Blue Alliance controller mode
// #define SGM // Shield Generator controller mode
// #define STM // Scoring Table controller mode


#define global static constexpr // Constant experssion accessable to all (cannot be modified)

/**
 * Config for Power port
 */
#ifdef RAM
#define IP "10.0.100.4"
global int DISPLAY_SDA_PORT = 14;
global int DISPLAY_SCL_PORT = 15;
#endif

#ifdef BAM
#define IP "10.0.100.5"
global int DISPLAY_SDA_PORT = 14;
global int DISPLAY_SCL_PORT = 15;
#endif


/**
 * Shield generator config
 */
#ifdef SGM
#define IP "10.0.100.3"
global int DISPLAY_SDA_PORT = 14;
global int DISPLAY_SCL_PORT = 15;
#endif

/**
 * Scoring table config
 */
#ifdef STM
#define IP "10.0.100.2"
global int DISPLAY_SDA_PORT = 14;
global int DISPLAY_SCL_PORT = 15;
#endif