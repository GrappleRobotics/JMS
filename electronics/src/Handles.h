#ifndef HANDLES_H
#define HANDLES_H
/**
 * Generic Handle for controller code
 * Returns 1 or 0 depending on error catch
 */
#define Handle(x) try {x return 0;}catch(const std::exception& e) {std::cout << e.what() << '\n';return 1;}

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
			x controller; \
			int programValue = controller.init(argc, argv); \
			if (programValue != 1) { \
				std::cout << "Program Start Error: " << programValue << std::endl; \
				return 1; \
			} \
			while (RUNNING) { \
				programValue = controller.update(argc, argv); \
				if (programValue != 0) { \
					std::cout << "Program Runtime Error: " << programValue << std::endl; \
					RUNNING = false; \
				} \
			} \
			return programValue != 0 ? 1 : 0; \
		} catch(const std::exception& e) { \
			std::cerr << e.what() << '\n'; \
			return 1; \
		} \
	}

#endif