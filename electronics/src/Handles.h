#ifndef HANDLES_H
#define HANDLES_H

/**
 * Handlers:
 * 
 * Handles wrap arround main code and sub code.
 * Will use try catch blocks to return 0 or 1 depending on outcome
 * 
 */

/**
 * Generic Handle for code
 * Returns 0 when code runs succesful
 * Returns 1 depending on error catch
 */
#define Handle(x) try {x return 0;} catch (const std::exception& e) {std::cout << e.what() << '\n';return 1;}


/**
 * Generic Handle for code
 * Returns 1 depending on error catch
 */
#define Handle_NO_RETURN_ON_SUCCESS(x) try {x} catch (const std::exception& e) {std::cout << e.what() << '\n';return 1;}


/**
 * Generic Handle for code
 * Does not return 0 or 1 regardless of catch
 */
#define Handle_NO_RETURN(x) try {x} catch (const std::exception& e) {std::cout << e.what() << '\n';}


/**
 * Generic Handle for looping controller code
 * Returns 1 or 0 depending on error catch
 * 
 * x being condition to end loop
 * y being code to execute in loop
 */
#define LoopHandle(x, y) while (x) { Handle_NO_RETURN_ON_SUCCESS(y) }


/**
 * Handle element (init or update) with integer return
 * Elements must all have a returner
 */
#define HandleElement(x) int elementValue = x; if (elementValue != 0) { std::cout << "Element Error"; return elementValue != 0 ? 1 : 0; }


/**
 * Handler for main controllers,
 * (insert interrupt code)
 */
#define HandleController(Controller) \
	int main(int argc, char const *argv[]) { \
		std::cout << "Program Start" << std::endl; \
		Controller controller; \
		int programValue = controller.start(argc, argv); \
		if (programValue != 0) { \
			std::cout << "Program Start Error: " << programValue << std::endl; \
		} \
		std::cout << "Program Exit: " << programValue << std::endl; \
		return programValue != 0 ? 1 : 0; \
	}
#endif