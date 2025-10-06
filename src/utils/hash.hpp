#ifndef UTILS_HASH_HPP
#define UTILS_HASH_HPP

#include "./dt.hpp"

namespace Utils {
  namespace Hash {
    inline DT::hash_t hash(const std::string&);
    constexpr DT::hash_t hash_cxpr(
      const char* str, 
      DT::hash_t res1 = 0, DT::hash_t res2 = 0, 
      DT::hash_t base1 = 17, DT::hash_t base2 = 61, 
      DT::hash_t M = 1e9 + 7) {
      return *str ? 
        hash_cxpr(str + 1, (res1 + (*str)) * base1 % M, (res2 + (*str)) * base2 % M, base1, base2, M) :
        res1 ^ res2;
    }
  }
}

inline Utils::DT::hash_t Utils::Hash::hash(const std::string& str) {
  constexpr DT::hash_t base1 = 17, base2 = 61, M = 1e9 + 7;
  DT::hash_t res1 = 0, res2 = 0;
  for(auto& c : str) {
    res1 += c; res1 *= base1;
    res2 += c; res2 *= base2;
    if(res1 >= M) res1 %= M;
    if(res2 >= M) res2 %= M;
  }
  return res1 ^ res2;
}

#endif