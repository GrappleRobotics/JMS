#include "Controllers/RAM_BAM/RAM_BAM.h"

int RAM_BAM::start(int argc, char const *argv[], int &userButton) {
	Handle(
		bool RUNNING = true;

		PowerPort pp;
		HandleElement(pp.init(argc, argv, userButton));
		
		LoopHandle(RUNNING,
			thread_sleep_for(5);
			HandleElement(pp.update(argc, argv, userButton));
		)
	)
}