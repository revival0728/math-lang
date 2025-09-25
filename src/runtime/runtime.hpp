#ifndef RUNTIME_HPP
#define RUNTIME_HPP

// TODO: Add user defined function support
// TODO: update Runtime declaration and implementation

#include <vector>
#include <string>
#include <utility>
#include <any>
#include "../utils.hpp"

class Runtime {
  private:
  using Idnt = Utils::BC::Idnt;
  using number_t = Utils::DT::number_t;
  using number_p = Utils::DT::number_p;
  using func_t = Utils::DT::func_t;
  using func_p = Utils::DT::func_p;

  public:
  using RtResult = Utils::Pipline::RtResult;

  protected:
  Idnt pre_value;
  std::vector<MemUnit> mem;
  bool has_idnt(int);
  std::pair<bool, const number_p> get_idnt_value(const Idnt&);
  std::pair<bool, const func_p> get_idnt_func(const Idnt&);
  
  public:
  Runtime();
  RtResult run(const Utils::Pipline::CmplResult&);

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const Runtime&);
  #endif
};

#endif
