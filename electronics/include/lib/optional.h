template<typename T>
class optional {
 public:
  optional() : _present{false} {}
  optional(T val) : _present{true} {
    _storage._value = val;
  }

  ~optional() {
    if (_present) _storage._value.T::~T();
  }

  bool has_value() const {
    return _present;
  }

  void clear() {
    if (_present)
      _storage._value.T::~T();
    _present = false;
  }

  void set(const T &other) {
    clear();
    _storage._value = other;
    _present = true;
  }

  T &get() {
    return _storage._value;
  }

  const T& get() const {
    return _storage._value;
  }

 private:
  union storage_t {
    unsigned char _nothing;
    T _value;
  };

  storage_t _storage;
  bool _present = false;
};