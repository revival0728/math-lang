#ifndef UTILS_PIPLINE_HPP
#define UTILS_PIPLINE_HPP

#include "./bc.hpp"
#include "./string.hpp"
#include <string>

namespace Utils {
  namespace Pipline {
    struct PrResult {
      Utils::BC::InstList inst_list;
      int mtable_size;
    };
    using CmplResult = PrResult;

    struct RtResult {
      enum ExitCode {
        Ok,
        Error,
        InvalidUse,
        UndefinedVar,
        UndefinedFunc,
        Null,
      } code;
      std::string msg;
      RtResult() {}
      RtResult(ExitCode _code, const std::string& _msg) : code(_code), msg(_msg) {}
      template<class ...P> RtResult(ExitCode _code, P... t) : 
        code(_code), 
        msg(Utils::String::bs(t...)) {}
      static RtResult make_null() { return RtResult(RtResult::Null, ""); }
      static RtResult make_corrupted_error() { return RtResult(RtResult::Error, "Corrupted bytecode instruction"); }
      static RtResult make_unknown_error() { return RtResult(RtResult::Error, "Unknown Error"); }
    };
  }
}

#endif
