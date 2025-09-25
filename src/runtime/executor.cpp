#include "runtime_base.hpp"

// TODO: add new instructions

void executor(Frame& frame, const Utils::BC::Instruction& inst) {
  using namespace Utils;
  using RtResult = Frame::RtResult;
  using Operator = BC::Operator;
  using Idnt = BC::Idnt;

  if(inst.oper < Grammer::ALL_OPER_NAMES_LEN) {
    Debug::console << "Runtime: handling [" << Grammer::ALL_OPER_NAMES[inst.oper] << "] instruction\n";
  }
  switch(inst.oper) {
  case Operator::set: {
    Idnt dest = inst.idnts[0], source = inst.idnts[1];
    auto [vs_ok, m_vs] = frame.get_munit(dest.frame_id, dest.idnt_id());
    if(!vs_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    if(m_vs.get<Null>()->is_valid()) {
      if(frame.rt_result().code == RtResult::UndefinedVar)
        frame.emplace_rt_result(RtResult::UndefinedVar, "Cannot assign an undefined value to a variable.");
      return;
    }
    auto [vd_ok, m_vd] = frame.get_munit(dest.frame_id, dest.idnt_id());
    if(!vd_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    m_vd.set(m_vs.get<Object>());
    break;
  }
  #define HANDLE_BIN_INST(INST_T, OPER) \
    case INST_T: {\
      Idnt ia = inst.idnts[0], ib = inst.idnts[1]; \
      auto [va_ok, va] = frame.get_munit(ia.frame_id, ia.idnt_id()); \
      auto [vb_ok, vb] = frame.get_munit(ib.frame_id, ib.idnt_id()); \
      auto [pv_ok, pv] = frame.get_munit(frame.frame_id(), -1); \
      if(!va_ok || !vb_ok || !pv_ok) { frame.set_rt_result(RtResult::make_corrupted_error()); return; } \
      pv.set(*va.get<Number>() OPER *vb.get<Number>()); \
      break; \
    }
  HANDLE_BIN_INST(Operator::plus, +)
  HANDLE_BIN_INST(Operator::multiply, *)
  HANDLE_BIN_INST(Operator::minus, -)
  HANDLE_BIN_INST(Operator::divide, /)
  case Operator::func: {
    Idnt func = inst.idnts.back();
    auto [fp_ok, m_fp] = frame.get_munit(func.frame_id, func.idnt_id());
    if(!fp_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    auto fp = m_fp.get<Callable>();
    if(!fp->is_valid()) {
      frame.emplace_rt_result(RtResult::InvalidUse, "Numbers are not callable object.");
      return;
    }
    auto pframe = frame.enter_new_frame();
    int arg_id = 0;
    for(auto it = std::next(inst.idnts.crbegin()), end = inst.idnts.crend(); it != end; ++it) {
      auto [pp_ok, m_pp] = frame.get_munit(it->frame_id, it->idnt_id_const());
      if(!pp_ok) {
        frame.set_rt_result(RtResult::make_corrupted_error());
        return;
      }
      auto pp = m_pp.get<Object>();
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
    if(!ppv_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    ppv.set(ppv.get<Number>());
    break;
  }
  case Operator::print: {
    Idnt idnt = inst.idnts[0];
    auto [vi_ok, m_vi] = frame.get_munit(idnt.frame_id, idnt.idnt_id());
    if(!vi_ok) {
      frame.set_rt_result(RtResult::make_corrupted_error());
      return;
    }
    auto vi = m_vi.get<Number>();
    if(!vi->is_valid()) {
      if(frame.rt_result().code == RtResult::UndefinedVar)
        frame.emplace_rt_result(RtResult::UndefinedVar, "Cannot print an undefined value.");
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