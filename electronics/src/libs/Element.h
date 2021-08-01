class Element {
 public:
	virtual ~Element() = default;

	virtual int init(int argc, char const *argv[], int &userButton);
	virtual int update(int argc, char const *argv[], int &userButton);
};