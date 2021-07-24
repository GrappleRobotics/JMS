#include "Controllers/RAM_BAM/RAM_BAM.h"

int RAM_BAM::start(int argc, char const *argv[], int &userButton) {
	HandleInterrupt(
		bool RUNNING = true;

		PowerPort pp;
		HandleElement(pp.init(argc, argv, userButton));
		
		LoopHandleInterrupt(RUNNING,
			// check if button is pushed, interrupt and execute 
			thread_sleep_for(5);
			HandleElement(pp.update(argc, argv, userButton));
			
			,
			/**
			 * Interrupt code
			 * @TODO
			 */
		),

		/**
		 * Interrupt code
		 * @TODO
		 */
	)
}