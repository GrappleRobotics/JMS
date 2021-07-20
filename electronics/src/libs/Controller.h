class Controller {
 public:
	virtual ~Controller() = default;

	virtual int start(int argc, char const *argv[], int &userButton);
};