#include "runtime.hpp"
#include "mathlib.hpp"
#include <typeinfo>

using namespace Utils;

Runtime::Runtime() {}

Runtime::RtResult Runtime::run(const Pipline::CmplResult& cmpl_res) {
  frame.get_mtable()->resize(cmpl_res.mtable_size);
  for(auto& inst : cmpl_res.inst_list) {
    executor(frame, inst);
    if(frame.rt_result().code != RtResult::Ok) {
      return frame.rt_result();
    }
  }
  return frame.rt_result();
}

#ifdef DEBUG
std::ostream& operator<<(std::ostream& os, const Runtime& rt) {
  using Idnt = Parser::Idnt;

  os << "[Runtime]:\n";
  os << "rt_result:\n";
  os << CLI::RT_RESULT_CODE[rt.rt_result.code] << ": " << rt.rt_result.msg << '\n';
  os << "pre_value: ";
  switch(rt.pre_value.idnt_type) {
  case Idnt::Raw:
    os << rt.pre_value.raw_value_const();
    break;
  case Idnt::None:
    os << "None";
    break;
  case Idnt::PreValue:
    os << "PreValue";
    break;
  case Idnt::Var:
    os << "Var(" << rt.pre_value.idnt_id_const() << ")";
    break;
  case Idnt::Func:
    os << "Func(" << rt.pre_value.idnt_id_const() << ")";
    break;
  }
  os << '\n';
  os << "mem:\n";
  for(int i = 0; i < rt.mem.size(); ++i)
    os << i << ": " << rt.mem[i] << '\n';
  os << "[Runtime END]";
  return os;
}
#endif
