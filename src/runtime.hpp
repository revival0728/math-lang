#ifndef RUNTIME_HPP
#define RUNTIME_HPP

#include <vector>
#include <string>
#include <utility>
#include <any>
#include <iostream>
#include "compiler.hpp"
#include "utils.hpp"

class RtMemUnit {
  public:
  using Idnt = Parser::CalcStep::Idnt;
  using number_t = MathLangUtils::DT::number_t;
  using number_p = MathLangUtils::DT::number_p;
  using func_t = MathLangUtils::DT::func_t;
  using func_p = MathLangUtils::DT::func_p;

  protected:
  std::shared_ptr<std::any> mem;

  public:
  RtMemUnit();
  template<class AnyT> std::pair<bool, const std::shared_ptr<AnyT>> get() const;
  template<class AnyT> void set(const AnyT&);

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const RtMemUnit&);
  #endif
};

class Runtime {
  private:
  using Idnt = Parser::CalcStep::Idnt;
  using number_t = MathLangUtils::DT::number_t;
  using number_p = MathLangUtils::DT::number_p;
  using func_t = MathLangUtils::DT::func_t;
  using func_p = MathLangUtils::DT::func_p;

  public:
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
      msg(MathLangUtils::String::bs(t...)) {}
    static RtResult make_null() { return RtResult(RtResult::Null, ""); }
    static RtResult make_corrupted_error() { return RtResult(RtResult::Error, "Corrupted bytecode instruction"); }
    static RtResult make_unknown_error() { return RtResult(RtResult::Error, "Unknown Error"); }
  } rt_result;

  protected:
  Idnt pre_value;
  std::vector<RtMemUnit> mem;
  bool has_idnt(int);
  std::pair<bool, const number_p> get_idnt_value(const Idnt&);
  std::pair<bool, const func_p> get_idnt_func(const Idnt&);
  
  public:
  Runtime();
  RtResult run(const Compiler::CmplResult&);

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const Runtime&);
  #endif
};

#endif
