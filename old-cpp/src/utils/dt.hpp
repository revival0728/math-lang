#ifndef UTILS_DT_HPP
#define UTILS_DT_HPP

#include <vector>
#include <memory>
#include <functional>

namespace Utils {
  namespace DT {
    using number_t = double;
    using number_p = std::shared_ptr<number_t>;
    using args_t = std::vector<number_p>;
    using raw_func_t = number_t(args_t&);
    using raw_func_p = number_t(*)(args_t&);
    using func_t = std::function<raw_func_t>;
    using func_p = std::shared_ptr<func_t>;
    using hash_t = int64_t;
    // every bit represents expected operator or idnt
    // the order of bit equals to the reverse order of ALL_OPER
    // the last bit of it represents idnt
    using exprsybit_t = uint16_t;
    template<class T> using SafeRet = std::pair<bool, T>;
  }
}

#endif