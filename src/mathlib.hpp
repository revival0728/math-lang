#ifndef MATHLIB_HPP
#define MATHLIB_HPP

#include "utils.hpp"
#include <vector>
#include <map>

#define _USE_MATH_DEFINES
#include <cmath>

namespace MathLangLib {
  using number_t = MathLangUtils::number_t;
  using func_p = MathLangUtils::func_p;
  using func_t = MathLangUtils::func_t;
  using args_t = MathLangUtils::args_t;

  #define WRAP_CPP_BUILTIN_FN_1(FN_NAME, _FN_NAME) \
    static number_t _FN_NAME(number_t v1) { return std::FN_NAME(v1); } \
    static number_t FN_NAME(args_t& args) { return MathLangUtils::Function::call_func(_FN_NAME, args); }
  #define WRAP_CPP_BUILTIN_FN_2(FN_NAME, _FN_NAME) \
    static number_t _FN_NAME(number_t v1, number_t v2) { return std::FN_NAME(v1, v2); } \
    static number_t FN_NAME(args_t& args) { return MathLangUtils::Function::call_func(_FN_NAME, args); }

  WRAP_CPP_BUILTIN_FN_1(cos, _cos)
  WRAP_CPP_BUILTIN_FN_1(sin, _sin)
  WRAP_CPP_BUILTIN_FN_1(tan, _tan)
  WRAP_CPP_BUILTIN_FN_1(acos, _acos)
  WRAP_CPP_BUILTIN_FN_1(asin, _asin)
  WRAP_CPP_BUILTIN_FN_1(atan, _atan)
  WRAP_CPP_BUILTIN_FN_1(abs, _abs)
  WRAP_CPP_BUILTIN_FN_1(sqrt, _sqrt)
  WRAP_CPP_BUILTIN_FN_1(cbrt, _cbrt)
  WRAP_CPP_BUILTIN_FN_1(ceil, _ceil)
  WRAP_CPP_BUILTIN_FN_1(floor, _floor)
  WRAP_CPP_BUILTIN_FN_1(trunc, _trunc)
  WRAP_CPP_BUILTIN_FN_1(round, _round)
  WRAP_CPP_BUILTIN_FN_2(pow, _pow)
  WRAP_CPP_BUILTIN_FN_1(exp, _exp)
  WRAP_CPP_BUILTIN_FN_2(fmod, _fmod)

  const std::unordered_map<std::string, const number_t> builtin_constant = {
    {"pi", M_PI},
    {"e",  M_E},
  };
  #define LIST_BUILTIN_FN(NAME, CPP_NAME) {NAME, MathLangLib::CPP_NAME}
  const std::unordered_map<std::string, const func_t> builtin_fn = {
    LIST_BUILTIN_FN("cos", cos),
    LIST_BUILTIN_FN("sin", sin),
    LIST_BUILTIN_FN("atan", tan),
    LIST_BUILTIN_FN("acos", acos),
    LIST_BUILTIN_FN("asin", asin),
    LIST_BUILTIN_FN("atan", atan),
    LIST_BUILTIN_FN("abs", abs),
    LIST_BUILTIN_FN("sqrt", sqrt),
    LIST_BUILTIN_FN("cbrt", cbrt),
    LIST_BUILTIN_FN("ceil", ceil),
    LIST_BUILTIN_FN("floor", floor),
    LIST_BUILTIN_FN("trunc", trunc),
    LIST_BUILTIN_FN("round", round),
    LIST_BUILTIN_FN("pow", pow),
    LIST_BUILTIN_FN("exp", exp),
    LIST_BUILTIN_FN("mod", fmod),
  };
}

#endif