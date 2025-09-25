#include "utils.hpp"
#include "runtime/mathlib.hpp"
#include <cassert>
#include <cctype>
#include <string>
#include <stack>

using namespace Utils;

bool Utils::Grammer::is_invalid(DT::exprsybit_t expect, DT::exprsybit_t found) { 
  return ((expect | found) ^ expect) != 0; 
}

DT::exprsybit_t make_exprsybit(std::initializer_list<BC::Operator> opers) {
  DT::exprsybit_t ret = 0;
  for(auto& oper : opers) {
    if(oper == BC::Operator::null) ret |= 1;
    else ret |= Grammer::OPER_BIT[oper];
  }
  return ret;
}

bool Utils::String::is_operator(const std::string& str) {
  for(auto& oper : Grammer::ALL_OPER) {
    if(oper == str) return true;
  }
  return false;
}

bool Utils::String::is_number(const std::string& str) {
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

Utils::DT::number_t Utils::String::to_number(const std::string& str) {
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

Utils::DT::hash_t Utils::Hash::hash(const std::string& str) {
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

void Utils::String::strip_self(std::string& str) {
  int i = 0;
  while(i < str.size() && std::isspace(str[i])) ++i;
  str.replace(str.cbegin(), str.cbegin() + i, "");
  i = str.size() - 1;
  while(i >= 0 && std::isspace(str[i])) --i;
  str.replace(str.cbegin() + i + 1, str.cend(), "");
}

std::string Utils::String::strip(const std::string& str) {
  std::string ret = str;
  Utils::String::strip_self(ret);
  return ret;
}
