#ifndef UTILS_GRAMMER_HPP
#define UTILS_GRAMMER_HPP

#include "dt.hpp"
#include <string>
#include <cstddef>

namespace Utils {
  namespace Grammer {
    const std::string ALL_KEYWORD[] = {"func", "end"};
    const std::string ALL_OPER[] = {"=","+","*","-","/","(",")",","};
    const std::string ALL_OPER_NAMES[] = {"set"      , "plus"     , "multiply" , "minus"    ,"divide"    , "lparen"   , "rparen"   , "argsplit"  , "null", "func", "print", "callbf", "def", "ret"};
    const int OPER_RANK[]              = {0          , 2          , 3          , 2          , 3          , 5          , 5          , 1           , 0     , 4  };
    const DT::exprsybit_t OPER_BIT[]   = {0b100000000, 0b010000000, 0b001000000, 0b000100000, 0b000010000, 0b000001000, 0b000000100, 0b000000010 , 0b000000001};
    constexpr std::size_t ALL_OPER_LEN = sizeof(ALL_OPER) / sizeof(std::string);
    constexpr std::size_t OPER_RANK_LEN = sizeof(OPER_RANK) / sizeof(int);
    constexpr std::size_t ALL_OPER_NAMES_LEN = sizeof(ALL_OPER_NAMES) / sizeof(std::string);
    constexpr std::size_t OPER_BIT_LEN = sizeof(OPER_BIT) / sizeof(DT::exprsybit_t);
    static_assert(ALL_OPER_LEN + 2 == OPER_RANK_LEN, "Rank count must equals to Operator count");
    static_assert(ALL_OPER_LEN + 6 == ALL_OPER_NAMES_LEN, "Operator Name count must equals to Full Operator count");
    static_assert(OPER_BIT_LEN + 5 == ALL_OPER_NAMES_LEN, "Operator Name count must equals to Full Operator count");
    inline bool is_invalid(DT::exprsybit_t, DT::exprsybit_t); 
  }
}

inline bool Utils::Grammer::is_invalid(DT::exprsybit_t expect, DT::exprsybit_t found) { 
  return ((expect | found) ^ expect) != 0; 
}

#endif