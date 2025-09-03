#include "utils.hpp"
#include <cassert>
#include <cctype>
#include <stack>
#include <string>
#include <cmath>
#include <iostream>

bool MathLangUtils::is_str_operator(const std::string& str) {
  for(auto& oper : MathLangUtils::ALL_OPER) {
    if(oper == str) return true;
  }
  return false;
}

bool MathLangUtils::is_str_number(const std::string& str) {
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

MathLangUtils::number_t MathLangUtils::pow(MathLangUtils::number_t a, MathLangUtils::number_t b) {
  return std::pow(a, b);
}

MathLangUtils::number_t MathLangUtils::str_to_number(const std::string& str) {
  auto c_str = str.c_str();
  char *pos = nullptr;
  std::stack<number_t> expo;
  do {
    number_t val = std::strtod(c_str, &pos);
    if(*pos == '^') {
      ++pos;
    }
    expo.push(val);
    c_str = pos;
  } while(*pos != '\0');
  assert(!expo.empty());
  number_t ret = expo.top(); expo.pop();
  while(!expo.empty()) {
    ret = MathLangUtils::pow(expo.top(), ret);
    expo.pop();
  }
  return ret;
}

MathLangUtils::hash_t MathLangUtils::hash(const std::string& str) {
  constexpr MathLangUtils::hash_t base1 = 17, base2 = 61, M = 1e9 + 7;
  MathLangUtils::hash_t res1 = 0, res2 = 0;
  for(auto& c : str) {
    res1 += c; res1 *= base1;
    res2 += c; res2 *= base2;
    if(res1 >= M) res1 %= M;
    if(res2 >= M) res2 %= M;
  }
  return res1 ^ res2;
}

void MathLangUtils::String::strip_self(std::string& str) {
  int i = 0;
  while(i < str.size() && std::isspace(str[i])) ++i;
  str.replace(str.cbegin(), str.cbegin() + i, "");
  i = str.size() - 1;
  while(i >= 0 && std::isspace(str[i])) --i;
  str.replace(str.cbegin() + i + 1, str.cend(), "");
}

std::string MathLangUtils::String::strip(const std::string& str) {
  std::string ret = str;
  MathLangUtils::String::strip_self(ret);
  return ret;
}