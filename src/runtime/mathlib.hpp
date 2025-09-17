#ifndef MATHLIB_HPP
#define MATHLIB_HPP

#include "../utils.hpp"
#include <cmath>

namespace MathLangLib {
  using namespace Utils::DT;

  #define MC_PI 3.14159265358979323846264338327950288
  #define MC_E  2.71828182845904523536028747135266250
  const std::unordered_map<std::string, const number_t> builtin_constant = {
    {"pi", MC_PI},
    {"e",  MC_E},
  };

  #define WRAP_CPP_BUILTIN_FN_1(FN_NAME, _FN_NAME) \
    static number_t _FN_NAME(number_t v1) { return std::FN_NAME(v1); } \
    static number_t FN_NAME(args_t& args) { return Utils::Function::call_func(_FN_NAME, args); }
  #define WRAP_CPP_BUILTIN_FN_2(FN_NAME, _FN_NAME) \
    static number_t _FN_NAME(number_t v1, number_t v2) { return std::FN_NAME(v1, v2); } \
    static number_t FN_NAME(args_t& args) { return Utils::Function::call_func(_FN_NAME, args); }

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
  WRAP_CPP_BUILTIN_FN_1(log10, _log10)
  WRAP_CPP_BUILTIN_FN_1(log2, _log2)
  WRAP_CPP_BUILTIN_FN_1(log, _log)
  WRAP_CPP_BUILTIN_FN_2(fmod, _fmod)

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
    LIST_BUILTIN_FN("log", log10),
    LIST_BUILTIN_FN("lg", log2),
    LIST_BUILTIN_FN("ln", log),
    LIST_BUILTIN_FN("mod", fmod),
  };
}

#endif
