#include <stdint.h>

template< class T > struct remove_reference      { typedef T type; };
template< class T > struct remove_reference<T&>  { typedef T type; };
template< class T > struct remove_reference<T&&> { typedef T type; };
template <class T> T&& forward(typename remove_reference<T>::type& t);
template <class T> T&& forward(typename remove_reference<T>::type&& t);

// Parts from  https://ojdip.net/2013/10/implementing-a-variant-type-in-cpp/
template<typename T, typename... Ts>
struct variant_helper {
  static const size_t size = sizeof(T) > variant_helper<Ts...>::size ? sizeof(T) : variant_helper<Ts...>::size;

  inline static void destroy(size_t tag, void *data) {
    if (tag == T::tag)
      reinterpret_cast<T*>(data)->~T();
    else
      variant_helper<Ts...>::destroy(tag, data);
  }

  inline static void copy(size_t old_t, const void *old_v, void *new_v) {
    if (old_t == T::tag)
      new (new_v) T(*reinterpret_cast<const T*>(old_v));
    else
      variant_helper<Ts...>::copy(old_t, old_v, new_v);
  }
};

template<typename T>
struct variant_helper<T>  {
  static const size_t size = sizeof(T);

  inline static void destroy(size_t tag, void *data) {
    if (tag == T::tag) reinterpret_cast<T*>(data)->~T();
  }

  inline static void copy(size_t old_t, const void *old_v, void *new_v) {
    if (old_t == T::tag)
      new (new_v) T(*reinterpret_cast<const T*>(old_v));
  }
};

template<unsigned int size>
class raw_data { char data[size]; };

template<typename... Ts>
struct tagged_variant {
  tagged_variant() { _tag = 0; }

  tagged_variant(tagged_variant<Ts...>&& old) : _tag(old._tag), _raw(old._raw) {
    old._tag = 0;
  }

  tagged_variant(const tagged_variant<Ts...>& old) : _tag(old._tag) {
    helper::copy(old._tag, &old._raw, &_raw);
  }

  template<class T>
  tagged_variant(const T& value) {
    new (&_raw) T(value);
    _tag = T::tag;
  }

  template<typename T>
  tagged_variant<Ts...>& operator=(const T& variant) {
    new (&_raw) T(variant);
    _tag = T::tag;

    return *this;
  }

  tagged_variant<Ts...>& operator= (tagged_variant<Ts...>&& old) {
    _raw = old._raw;
    _tag = old._tag;
    old._tag = 0;

    return *this;
  }

  ~tagged_variant() { helper::destroy(_tag, &_raw); }

  template<typename T>
  bool is() const {
    return _tag == T::tag;
  }

  template<typename T>
  T& get() {
    // DANGER
    return *reinterpret_cast<T*>(&_raw);
  }

  template<typename T>
  const T& get() const {
    // DANGER (i live dangerously bitch)
    return *reinterpret_cast<const T*>(&_raw);
  }

  size_t tag() const {
    return _tag;
  }

 private:
  typedef variant_helper<Ts...> helper;

  size_t _tag;
  raw_data<helper::size> _raw;
};