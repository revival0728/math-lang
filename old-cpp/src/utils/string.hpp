#ifndef UTILS_STRING_HPP
#define UTILS_STRING_HPP

#include "./dt.hpp"
#include "./grammer.hpp"
#include "../mathlib.hpp"
#include <string>
#include <stack>
#include <cassert>

namespace Utils {
  namespace String {
    #define __TO_STRING(M) #M
    #define TO_STRING(M) __TO_STRING(M)
    template<class T> inline std::string to_string(const T[]);
    template<class T> inline std::string to_string(const T&);
    template<> inline std::string to_string<std::string>(const std::string&);
    template<> inline std::string to_string<DT::exprsybit_t>(const DT::exprsybit_t&);
    template<class T> inline std::string bs(const T&);
    template<class T, class ...P> inline std::string bs(const T&, const P&...);
    inline bool is_operator(const std::string&);
    inline bool is_number(const std::string&);
    inline DT::number_t to_number(const std::string&);
    inline void strip_self(std::string&);
    inline std::string strip(const std::string&);
  }
}

template<> inline std::string Utils::String::to_string<char>(const char str[]) {
  return std::string(str);
}
template <class T> inline std::string Utils::String::to_string(const T &var) {
  return std::to_string(var);
}
template<> inline std::string Utils::String::to_string<std::string>(const std::string& str) {
  return str;
}
template<> inline std::string Utils::String::to_string<Utils::DT::exprsybit_t>(const Utils::DT::exprsybit_t& bits) {
  if(bits == 0) return "nothing";
  std::string str;
  if(bits & 1) str.append("identifier ");
  for(int i = 1; i < 9; ++i) {
    if(bits & (1 << i)) {
      str.append(Grammer::ALL_OPER[Grammer::ALL_OPER_LEN - i]);
      str.push_back(' ');
    }
  }
  return str;
}
template<class T> inline std::string Utils::String::bs(const T& var) {
  return Utils::String::to_string(var);
}
template<class T, class ...P> inline std::string Utils::String::bs(const T& var, const P&... t) {
  return Utils::String::to_string(var) += bs(t...);
}

inline bool Utils::String::is_operator(const std::string& str) {
  for(auto& oper : Grammer::ALL_OPER) {
    if(oper == str) return true;
  }
  return false;
}

inline bool Utils::String::is_number(const std::string& str) {
  bool in_num = false;
  bool has_dot = false;
  bool has_e = false;
  bool check_last = false;
  for(auto& c : str) {
    if('0' <= c && c <= '9') {
      in_num = true;
    }
    else if(c == '.') {
      if(!in_num || has_dot) return false;
      has_dot = true;
      check_last = true;
    }
    else if(c == 'e' || c == 'E') {
      if(!in_num || has_e) return false;
      has_e = true;
      check_last = true;
    }
    else if(c == '^') {
      if(!in_num) return false;
      check_last = true;
      in_num = has_dot = has_e = false;
    } else {
      return false;
    }
  }
  if(check_last && '0' <= str.back() && str.back() <= '9') return true;
  if(!check_last) return true;
  return false;
}

inline Utils::DT::number_t Utils::String::to_number(const std::string& str) {
  auto c_str = str.c_str();
  char *pos = nullptr;
  std::stack<DT::number_t> expo;
  do {
    DT::number_t val = std::strtod(c_str, &pos);
    if(*pos == '^') {
      ++pos;
    }
    expo.push(val);
    c_str = pos;
  } while(*pos != '\0');
  assert(!expo.empty());
  DT::number_t ret = expo.top(); expo.pop();
  while(!expo.empty()) {
    ret = MathLangLib::_pow(expo.top(), ret);
    expo.pop();
  }
  return ret;
}


inline void Utils::String::strip_self(std::string& str) {
  int i = 0;
  while(i < str.size() && std::isspace(str[i])) ++i;
  str.replace(str.cbegin(), str.cbegin() + i, "");
  i = str.size() - 1;
  while(i >= 0 && std::isspace(str[i])) --i;
  str.replace(str.cbegin() + i + 1, str.cend(), "");
}

inline std::string Utils::String::strip(const std::string& str) {
  std::string ret = str;
  Utils::String::strip_self(ret);
  return ret;
}

#endif