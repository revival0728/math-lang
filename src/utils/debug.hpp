#ifndef UTILS_DEBUG_HPP
#define UTILS_DEBUG_HPP

namespace Utils {
  namespace Debug {
    class Console {
      public:
        template<class T> friend Console& operator<<(Console& console, T val) {
          #ifdef DEBUG
            std::cerr << val;
            return console;
          #else
            return console;
          #endif
        }
    };
    static Console console;
  }
}

#endif