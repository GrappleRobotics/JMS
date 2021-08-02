class Controller {
 public:
	virtual ~Controller() = default;
	virtual int start(int argc, char const *argv[]);

	bool getRUNNING() {
		return RUNNING;
	}

	void setRUNNING(bool state) {
		RUNNING = state;
	}

 private:
	bool RUNNING = true;
};