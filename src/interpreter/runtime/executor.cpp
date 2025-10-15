#include "mathlib.hpp"
#include "runtime_base.hpp"
#include "utils/bc.hpp"
#include "utils/dt.hpp"
#include <cassert>
#include <vector>

// TODO: consider raw_value in Idnt

Utils::DT::SafeRet<ObjPtr<Object> const> get_value(Frame& frame, const Utils::BC::Idnt& idnt) {
  using Idnt = Utils::BC::Idnt;
  using SafeRet = Utils::DT::SafeRet<ObjPtr<Object>>;
  using RtResult = Frame::RtResult;

  assert(idnt.idnt_type != Idnt::Str);  // Idnt::Str are special idnt for builtin_fn
  switch(idnt.idnt_type) {
  case Idnt::Raw:
    return SafeRet(true, Number(idnt.raw_value_const()).to_ptr());
  case Idnt::Var: {
    auto [ok, m] = frame.get_munit(idnt.frame_id, idnt.idnt_id_const());
    if(!ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return SafeRet(false, nullptr);
    }
    return SafeRet(true, m.get<Object>());
  }
  case Idnt::Func: {
    auto [ok, m] = frame.get_munit(idnt.frame_id, idnt.idnt_id_const());
    if(!ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return SafeRet(false, nullptr);
    }
    return SafeRet(true, m.get<Object>());
  }
  case Idnt::PreValue: {
    auto [ok, m] = frame.get_munit(idnt.frame_id, -1);
    if(!ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return SafeRet(false, nullptr);
    }
    return SafeRet(true, m.get<Object>());
  }
  case Idnt::Str:
    frame.set_rt_result(RtResult::make_corrupted_error());
    return SafeRet(false, nullptr);
  case Idnt::None:
    frame.set_rt_result(RtResult::make_corrupted_error());
    return SafeRet(false, nullptr);
  }
}

Utils::DT::SafeRet<ObjPtr<Object> const> get_variable(Frame& frame, const Utils::BC::Idnt& idnt) {
  using Idnt = Utils::BC::Idnt;
  using SafeRet = Utils::DT::SafeRet<ObjPtr<Object>>;
  using RtResult = Frame::RtResult;

  if(idnt.idnt_type == Idnt::Var || idnt.idnt_type == Idnt::Func) {
    auto [ok, m] = frame.get_munit(idnt.frame_id, idnt.idnt_id_const());
    if(!ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return SafeRet(false, nullptr);
    }
    return SafeRet(true, m.get<Object>());
  }
  frame.emplace_rt_result(RtResult::UndefinedVar, "Cannot find an undefined variable (idnt_id: ", idnt.idnt_id_const(), ").",
                                                        "This may caused by trying to get an undefined variable or assigning value to a raw number.");
  return SafeRet(false, nullptr);
}

void executor(Frame& frame, const Utils::BC::Instruction& inst) {
  using namespace Utils;
  using RtResult = Frame::RtResult;
  using Operator = BC::Operator;
  using Idnt = BC::Idnt;

  auto fstate = frame.get_state();
  if(fstate.state == Frame::FState::decl) {
    auto [fp_ok, m_fp] = frame.get_munit(frame.frame_id(), fstate.decl_id);
    if(!fp_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    auto fp = m_fp.get<Callable>();
    if(!fp->is_valid()) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    fp->add_inst(inst);
    return;
  }
  if(inst.oper < Grammer::ALL_OPER_NAMES_LEN) {
    Debug::console << "Runtime: handling [" << Grammer::ALL_OPER_NAMES[inst.oper] << "] instruction\n";
  }
  assert(fstate.state == Frame::FState::exec);
  switch(inst.oper) {
  case Operator::set: {
    Idnt dest = inst.idnts[0], source = inst.idnts[1];
    auto [vs_ok, vs] = get_value(frame, source);
    if(!vs_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    auto null_vs = vs->cast_self<Null>();
    auto is_null = null_vs->is_valid();
    if(vs->cast_self<Null>()->is_valid()) {
      frame.emplace_rt_result(RtResult::UndefinedVar, "Cannot assign an undefined value to a variable.");
      return;
    }
    auto [vd_ok, m_vd] = frame.get_munit(dest.frame_id, dest.idnt_id());
    if(!vd_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    m_vd.set(vs);
    break;
  }
  #define HANDLE_BIN_INST(INST_T, OPER) \
    case INST_T: {\
      Idnt ia = inst.idnts[0], ib = inst.idnts[1]; \
      auto [va_ok, va] = get_value(frame, ia); \
      auto [vb_ok, vb] = get_value(frame, ib); \
      auto [pv_ok, pv] = frame.get_munit(frame.frame_id(), -1); \
      if(!va_ok || !vb_ok || !pv_ok) { frame.set_rt_result(RtResult::make_corrupted_error()); return; } \
      pv.set(*va->cast_self<Number>() OPER *vb->cast_self<Number>()); \
      break; \
    }
  HANDLE_BIN_INST(Operator::plus, +)
  HANDLE_BIN_INST(Operator::multiply, *)
  HANDLE_BIN_INST(Operator::minus, -)
  HANDLE_BIN_INST(Operator::divide, /)
  case Operator::def: {
    Idnt func_id = inst.idnts.front();
    Callable cobj(inst.idnts.size() - 1);
    auto [fp_ok, m_fp] = frame.get_munit(frame.frame_id(), func_id.idnt_id());
    if(!fp_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    m_fp.set(cobj);
    frame.set_state(Frame::FState::decl, func_id.idnt_id());
  }
  case Operator::ret: {
    Idnt idnt = inst.idnts[0];
    auto [vi_ok, vi] = frame.get_munit(idnt.frame_id, idnt.idnt_id());
    if(!vi_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    auto [ppv_ok, ppv] = frame.get_munit(frame.pframe().frame_id(), -1);
    if(!ppv_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    ppv.set(vi.get<Object>());
  }
  case Operator::func: {
    Idnt func = inst.idnts.back();
    auto [fp_ok, p_fp] = get_variable(frame, func);
    if(!fp_ok) {
      if(frame.rt_result().code == RtResult::UndefinedVar)
        frame.emplace_rt_result(RtResult::InvalidUse, "Trying to call a not callable value.");
      return;
    }
    auto fp = p_fp->cast_self<Callable>();
    if(!fp->is_valid()) {
      frame.emplace_rt_result(RtResult::InvalidUse, "Numbers are not callable object.");
      return;
    }
    if(fp->arg_cnt() != inst.idnts.size() - 1) {
      frame.emplace_rt_result(RtResult::InvalidUse, "Invalid use of function, expected ", fp->arg_cnt(), " arguments, found", inst.idnts.size() - 1);
      return;
    }
    auto pframe = frame.enter_new_frame();
    int arg_id = 0;
    for(auto it = std::next(inst.idnts.crbegin()), end = inst.idnts.crend(); it != end; ++it) {
      auto [pp_ok, pp] = get_value(frame, *it);
      if(!pp_ok) {
        frame.set_rt_result(RtResult::make_corrupted_error());
        return;
      }
      auto [arg_ok, m_arg] = frame.get_munit(frame.frame_id(), arg_id);
      if(!arg_ok) {
        frame.set_rt_result(RtResult::make_corrupted_error());
        return;
      }
      m_arg.set(pp);
      arg_id++;
    }
    fp->call(frame);
    auto [ppv_ok, ppv] = pframe.get_munit(pframe.frame_id(), -1);
    auto [pv_ok, pv] = get_value(frame, Idnt::make_pre_value(frame.frame_id()));
    if(!ppv_ok || !pv_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    ppv.set(pv);
    break;
  }
  case Operator::callbf: {
    auto [pv_ok, m_pv] = frame.get_munit(frame.frame_id(), -1);
    if(!pv_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    auto fn = MathLangLib::builtin_fn.find(inst.idnts.back().idnt_str_const());
    assert(fn != MathLangLib::builtin_fn.end());
    Utils::DT::args_t arg_list;
    for(auto it = std::next(inst.idnts.cbegin()); it != inst.idnts.cend(); ++it) {
      auto [iv_ok, p_iv] = get_value(frame, *it);
      if(!iv_ok) {
        frame.set_rt_result(RtResult::make_corrupted_error());
        return;
      }
      auto iv = p_iv->cast_self<Number>();
      if(!iv->is_valid()) {
        frame.emplace_rt_result(RtResult::InvalidUse, "Builtin functions can only call with Number-type.");
        return;
      }
      arg_list.push_back(iv->cast_data<Utils::DT::number_t>());
    }
    auto fn_ret = (fn->second)(arg_list);
    m_pv.set(Number(fn_ret));
  }
  case Operator::print: {
    Idnt idnt = inst.idnts[0];
    auto [vi_ok, p_vi] = get_value(frame, idnt);
    if(!vi_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    auto vi = p_vi->cast_self<Number>();
    if(!vi->is_valid()) {
      if(frame.rt_result().code == RtResult::UndefinedVar)
        frame.emplace_rt_result(RtResult::UndefinedVar, "Cannot print an undefined value.");
      else
        frame.emplace_rt_result(RtResult::InvalidUse, "Functions cannot be printed.");
      return;
    }
    frame.emplace_rt_result(RtResult::Ok, Utils::String::to_string(*vi->cast_data<Number::number_t>()));
    break;
  }
  default:
    frame.emplace_rt_result(RtResult::Error, "Uknown byecode instruction (bytecode=", inst.oper, ")");
    break;
  }
}
