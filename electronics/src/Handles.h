#ifndef HANDLES_H
#define HANDLES_H

static unsigned int interruptFlag = 1;

/**
 * Handlers:
 * 
 * Handles wrap arround main code and sub code.
 * Will use try catch blocks to return 0 or 1 depending on outcome,
 * and will check any interrupts, and switch to interrupt priority code before continuing on main code.
 * 
 */


/**
 * CODE INTERRUPTS // DO NOT CHANGE //
 */
#define INTERRUPT interruptFlag = 0
#define INTERRUPT_HANDLED interruptFlag = 1
#define INTERRUPT_HANDLER(handle, x, intCode) \
	switch (interruptFlag) { \
		case 0: \
			intCode \
			break; \
		case 1: \
			handle(x) \
			break; \
		default: \
			handle(x) \
			break; \
	} \

/**
 * Generic Handle for code
 * Returns 0 when code runs succesful
 * Returns 1 depending on error catch
 */
#define Handle(x) try {x return 0;} catch (const std::exception& e) {std::cout << e.what() << '\n';return 1;}
#define HandleWithInterrupt(x) INTERRUPT_HANDLER(Handle, x,)
#define HandleInterrupt(x, intCode) INTERRUPT_HANDLER(Handle, x, intCode)


/**
 * Generic Handle for code
 * Returns 1 depending on error catch
 */
#define Handle_NO_RETURN_ON_SUCCESS(x) try {x} catch (const std::exception& e) {std::cout << e.what() << '\n';return 1;}
#define HandleWithInterrupt_NO_RETURN_ON_SUCCESS(x) INTERRUPT_HANDLER(Handle_NO_RETURN_ON_SUCCESS, x,)
#define HandleInterrupt_NO_RETURN_ON_SUCCESS(x, intCode) INTERRUPT_HANDLER(Handle_NO_RETURN_ON_SUCCESS, x, intCode)


/**
 * Generic Handle for code
 * Does not return 0 or 1 regardless of catch
 */
#define Handle_NO_RETURN(x) try {x} catch (const std::exception& e) {std::cout << e.what() << '\n';}
#define HandleWithInterrupt_NO_RETURN(x) INTERRUPT_HANDLE(Handle_NO_RETURN, x,)
#define HandleInterrupt_NO_RETURN(x, intCode) INTERRUPT_HANDLE(Handle_NO_RETURN, x, intCode)


/**
 * Generic Handle for looping controller code
 * Returns 1 or 0 depending on error catch
 * 
 * x being condition to end loop
 * y being code to execute in loop
 */
#define LoopHandle(x, y) while (x) { Handle_NO_RETURN_ON_SUCCESS(y) }
#define LoopHandleWithInterrupt(x, y) while(x) { INTERRUPT_HANDLER(Handle_NO_RETURN_ON_SUCCESS, y,) }
#define LoopHandleInterrupt(x, y, intCode) while(x) { INTERRUPT_HANDLER(Handle_NO_RETURN_ON_SUCCESS, y, intCode) }


/**
 * Handle element (init or update) with integer return
 * Elements must all have a returner
 */
#define HandleElement(x) int elementValue = x; if (elementValue != 0) { std::cout << "Element Error"; return elementValue != 0 ? 1 : 0; }
#define HandleElementWithInterrupt(x) INTERRUPT_HANDLER(HandleElement, x,)
#define HandleElementInterrupt(x, intCode) INTERRUPT_HANDLER(HandleElement, x, intCode)


/**
 * Handler for main controllers,
 * (insert interrupt code)
 */
#define HandleController(Controller) \
	/* DigitalIn userButton(USER_BUTTON);*/ \
	int userButtonInt = 0; \
	bool running = false; \
	/*void userButtonUpdate() { while (running) {userButtonInt = userButton; } }*/ \
	int main(int argc, char const *argv[]) { \
		if (interruptFlag == 0) { \
			INTERRUPT_HANDLED; \
		} \
		std::cout << "Program Start" << std::endl; \
		running = true; \
		/*Thread userButtonUpdate_t;*/ \
		/*userButtonUpdate_t.start(userButtonUpdate);*/ \
		Controller controller; \
		int programValue = controller.start(argc, argv, userButtonInt); \
		if (programValue != 0) { \
			std::cout << "Program Start Error: " << programValue << std::endl; \
		} \
		std::cout << "Program Exit: " << programValue << std::endl; \
		running = false; \
		/*userButtonUpdate_t.join();*/ \
		return programValue != 0 ? 1 : 0; \
	}
#endif