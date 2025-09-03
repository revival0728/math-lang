#include "runtime.hpp"
#include "mathlib.hpp"
#include <typeindex>
#include <typeinfo>

RtMemUnit::RtMemUnit() { 
  mem = std::shared_ptr<std::any>();
}

template<class AnyT> std::pair<bool, const std::shared_ptr<AnyT>> RtMemUnit::get() const {
  if(!mem)
    return std::make_pair(false, std::shared_ptr<AnyT>());
  if(!mem->has_value())
    return std::make_pair(false, std::shared_ptr<AnyT>());
  if(mem->type() == typeid(AnyT)) {
    return std::make_pair(
      true, 
      std::shared_ptr<AnyT>(mem, std::any_cast<AnyT>(mem.get()))
    );
  }
  return std::make_pair(false, std::shared_ptr<AnyT>());
}

template<class AnyT> void RtMemUnit::set(const AnyT& value) {
  mem.reset(new std::any(value));
}

#ifdef DEBUG
std::ostream& operator<<(std::ostream& os, const RtMemUnit& rtm) {
  using number_t = RtMemUnit::number_t;
  using func_t = RtMemUnit::func_t;

  if(!rtm.mem) {
    return os << "nullptr";
  }
  if(!rtm.mem->has_value()) {
    return os << "Null";
  }
  if(rtm.mem->type() == typeid(number_t)) {
    return os << "Var->" << *(rtm.get<number_t>().second);
  }
  if(rtm.mem->type() == typeid(func_t)) {
    return os << "Func->" << (rtm.get<func_t>().second)->target<MathLangUtils::raw_func_p>();
  }
  return os;
}
#endif

Runtime::Runtime() {
  pre_value = Idnt::make_pre_value();
  mem = std::vector<RtMemUnit>();
  rt_result = RtResult(RtResult::Null, "");
}

bool Runtime::has_idnt(int idnt_id) {
  return idnt_id >= 0 && idnt_id < mem.size();
}

std::pair<bool, const Runtime::number_p> Runtime::get_idnt_value(const Runtime::Idnt& idnt) {
  switch (idnt.idnt_type) {
  case Idnt::PreValue:
    return {true, std::make_shared<number_t>(pre_value.raw_value)};
    break;
  case Idnt::Raw:
    return {true, std::make_shared<number_t>(idnt.raw_value)};
    break;
  case Idnt::Var: {
    if(!has_idnt(idnt.idnt_id)) {
      rt_result = RtResult(RtResult::UndefinedVar, "Undefined variable (Idnt_id: ", idnt.idnt_id, ")");
      return {false, nullptr};
    }
    auto value = mem[idnt.idnt_id].get<number_t>();
    if(!value.first) {
      rt_result = RtResult(RtResult::UndefinedVar, "Undefined variable (Idnt_id: ", idnt.idnt_id, ")");
      return {false, nullptr};
    }
    return {true, value.second};
    break;
  }
  case Idnt::Func:
    rt_result = RtResult(RtResult::InvalidUse, "Functions cannot be used as variable");
    return {false, nullptr};
    break;
  case Idnt::None:
    rt_result = RtResult::make_corrupted_error();
    return {false, nullptr};
  default:
    rt_result = RtResult::make_unknown_error();
    return {false, nullptr};
    break;
  }
  return {false, nullptr};
}

std::pair<bool, const Runtime::func_p> Runtime::get_idnt_func(const Runtime::Idnt& idnt) {
  switch (idnt.idnt_type) {
  case Idnt::PreValue:
    rt_result = RtResult::make_corrupted_error();
    return std::make_pair(false, func_p());
    break;
  case Idnt::Raw:
    rt_result = RtResult::make_corrupted_error();
    return std::make_pair(false, func_p());
    break;
  case Idnt::Var:
    rt_result = RtResult(RtResult::InvalidUse, "Variables cannot be used as function");
    return std::make_pair(false, func_p());
    break;
  case Idnt::Func: {
    if(!has_idnt(idnt.idnt_id)) {
      rt_result = RtResult(RtResult::UndefinedFunc, "Undefined function (Idnt_id: ", idnt.idnt_id, ")");
      return std::make_pair(false, func_p());
    }
    auto fp = mem[idnt.idnt_id].get<func_t>();
    if(!fp.first) {
      rt_result = RtResult(RtResult::InvalidUse, "Variables cannot be used as function");
      return std::make_pair(false, func_p());
    }
    return std::make_pair(true, fp.second);
    break;
  }
  case Idnt::None:
    rt_result = RtResult::make_corrupted_error();
    return std::make_pair(false, func_p());
  default:
    rt_result = RtResult::make_unknown_error();
    return std::make_pair(false, func_p());
    break;
  }
  return std::make_pair(false, func_p());
}

Runtime::RtResult Runtime::run(Parser::Result_CRef pr_result) {
  using Operator = Parser::CalcStep::Operator;

  rt_result = RtResult::make_null();

  // add idnts in nidnt_table to mem
  // add builtins
  mem.resize(pr_result.idnt_count, RtMemUnit());
  for(auto& [idnt, idnt_id] : pr_result.nidnt_table) {
    auto blfn_it = MathLangLib::builtin_fn.find(idnt);
    if(blfn_it != MathLangLib::builtin_fn.end()) {
      mem[idnt_id].set<func_t>(blfn_it->second);
    }
  }
  for(auto& [idnt, idnt_id] : pr_result.nidnt_table) {
    auto blct_it = MathLangLib::builtin_constant.find(idnt);
    if(blct_it != MathLangLib::builtin_constant.end()) {
      mem[idnt_id].set<number_t>(blct_it->second);
    }
  }

  for(auto& in : pr_result.calc_list) {
    if(in.oper < MathLangUtils::ALL_OPER_NAMES_LEN) {
      Debug::console << "Runtime: handling [" << MathLangUtils::ALL_OPER_NAMES[in.oper] << "] instruction\n";
    }
    switch(in.oper) {
    case Operator::set: {
      Idnt dest = in.idnts[0], source = in.idnts[1];
      if(!has_idnt(dest.idnt_id))
        return rt_result = RtResult::make_corrupted_error();
      auto vs = get_idnt_value(source);
      if(!vs.first) {
        if(rt_result.code == RtResult::UndefinedVar)
          rt_result = RtResult(RtResult::UndefinedVar, "Cannot assign an undefined value to a variable.");
        return rt_result;
      }
      mem[dest.idnt_id].set<number_t>(*vs.second);
      break;
    }
    #define HANDLE_BIN_INST(INST_T, OPER) \
      case INST_T: {\
        Idnt ia = in.idnts[0], ib = in.idnts[1]; \
        auto va = get_idnt_value(ia), vb = get_idnt_value(ib); \
        if(!va.first || !va.second) return rt_result; \
        pre_value = Idnt::make_raw(*va.second OPER *vb.second); \
        break; \
      }
    HANDLE_BIN_INST(Operator::plus, +)
    HANDLE_BIN_INST(Operator::multiply, *)
    HANDLE_BIN_INST(Operator::minus, -)
    HANDLE_BIN_INST(Operator::divide, /)
    case Operator::func: {
      Idnt func = in.idnts.back();
      auto fp = get_idnt_func(func);
      if(!fp.first) return rt_result;
      MathLangUtils::args_t args;
      for(auto it = std::next(in.idnts.crbegin()), end = in.idnts.crend(); it != end; ++it) {
        auto pp = get_idnt_value(*it);
        if(!pp.first) {
          rt_result = RtResult(RtResult::InvalidUse, "Functions cannot used as parameter.");
          return rt_result;
        }
        args.push_back(pp.second);
      }
      pre_value = Idnt::make_raw((*fp.second)(args));
      break;
    }
    case Operator::print: {
      Idnt idnt = in.idnts[0];
      auto vi = get_idnt_value(idnt);
      if(!vi.first) {
        if(rt_result.code == RtResult::UndefinedVar)
          rt_result = RtResult(RtResult::UndefinedVar, "Cannot print an undefined value.");
        return rt_result;
      }
      rt_result = RtResult(RtResult::Ok, MathLangUtils::String::to_string(*vi.second));
      break;
    }
    default:
      rt_result = RtResult(RtResult::Error, "Uknown byecode instruction (bytecode=", in.oper, ")");
      break;
    }
  }
  if(rt_result.code == RtResult::Null) {
    rt_result.code = RtResult::Ok;
  }
  return rt_result;
}

#ifdef DEBUG
std::ostream& operator<<(std::ostream& os, const Runtime& rt) {
  using Idnt = Parser::CalcStep::Idnt;

  os << "[Runtime]:\n";
  os << "rt_result:\n";
  os << MathLangUtils::RT_RESULT_CODE[rt.rt_result.code] << ": " << rt.rt_result.msg << '\n';
  os << "pre_value: ";
  switch(rt.pre_value.idnt_type) {
  case Idnt::Raw:
    os << rt.pre_value.raw_value;
    break;
  case Idnt::None:
    os << "None";
    break;
  case Idnt::PreValue:
    os << "PreValue";
    break;
  case Idnt::Var:
    os << "Var(" << rt.pre_value.idnt_id << ")";
    break;
  case Idnt::Func:
    os << "Func(" << rt.pre_value.idnt_id << ")";
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