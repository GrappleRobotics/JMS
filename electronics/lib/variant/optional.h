#include <new>

template<typename T>
class optional {
 public:
  optional() : _present{false} {}

  optional(const T &val) : _present{true} {
    // _storage._value = val;
    // *reinterpret_cast<T*>(_data) = val;
    new (reinterpret_cast<T*>(_data)) T(val);
  }

  optional(const optional<T> &other) : _present{other.has_value()} {
    if (other._present)
      new (reinterpret_cast<T*>(_data)) T(other.get());
      // *reinterpret_cast<T*>(_data) = other.get();
  }

  ~optional() {
    if (_present) reinterpret_cast<T*>(_data)->~T();
  }

  bool has_value() const {
    return _present;
  }

  void clear() {
    if (_present)
      reinterpret_cast<T*>(_data)->~T();
    _present = false;
  }

  void set(const T &other) {
    clear();
    new (_data) T(other);
    // _storage._value = other;
    _present = true;
  }

  T &get() {
    // return _storage._value;
    return *reinterpret_cast<T*>(_data);
  }

  const T& get() const {
    return *reinterpret_cast<const T*>(_data);
  }

 private:
  char _data[sizeof(T)];
  bool _present = false;
};