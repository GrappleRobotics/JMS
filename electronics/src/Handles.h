#ifndef HANDLES_H
#define HANDLES_H
/**
 * Generic Handle for controller code
 * Returns 1 or 0 depending on error catch
 */
#define Handle(x) try {x return 0;}catch(const std::exception& e) {std::cout << e.what() << '\n';return 1;}

/**
 * Generic Handle for looping controller code
 * Returns 1 or 0 depending on error catch
 * 
 * x being condition to end loop
 * y being code to execute in loop
 */
#define LoopHandle(x, y) while (x) { Handle(y) }

/**
 * Program status
 */

/**
 * Handler for main controllers
 */
#define HandleController(x) int main(int argc, char const *argv[]) { \
		try { \
			std::cout << "Program Start" << std::endl; \
			bool RUNNING = true; \
			DigitalIn userButton(USER_BUTTON); \
			x controller; \
			int programValue = controller.init(argc, argv, (int)userButton); \
			if (programValue != 0) { \
				std::cout << "Program Start Error: " << programValue << std::endl; \
				return 1; \
			} \
			while (RUNNING) { \
				programValue = controller.update(argc, argv, (int)userButton); \
				if (programValue != 0) { \
					std::cout << "Program Runtime Error: " << programValue << std::endl; \
					RUNNING = false; \
				} \
			} \
			std::cout << "Program Exit: " << programValue << std::endl; \
			return programValue != 0 ? 1 : 0; \
		} catch(const std::exception& e) { \
			std::cerr << e.what() << '\n'; \
			return 1; \
		} \
	}

#endif