#include "runtime_base.hpp"
#include <cassert>

Frame::Frame() {
  frame = std::make_shared<__Frame>();
  frame->frame_id = 0;
  frame->mtable = std::make_shared<MemTable>();
  frame->rt_result = std::make_shared<RtResult>(RtResult::make_null());
  frame->pframe = nullptr;
}

Frame::Frame(_Frame _frame) {
  frame = _frame;
}

int Frame::frame_id() {
  return frame->frame_id;
}

Frame Frame::get_frame(const std::size_t id) {
  assert(id >= 0 && id <= frame_id());
  _Frame ret = frame;
  for(std::size_t i = 0; i < frame_id() - id; ++i) {
    ret = ret->pframe;
  }
  return Frame(ret);
}

Frame Frame::pframe() const {
  return Frame(frame->pframe);
}

Frame Frame::enter_new_frame() {
  _Frame nframe;
  nframe->frame_id = frame->frame_id + 1;
  nframe->mtable = std::make_shared<MemTable>();
  nframe->rt_result = frame->rt_result;
  nframe->pframe = frame;
  frame = nframe;
  return pframe();
}

void Frame::back_to_parent() {
  frame = frame->pframe;
}

std::shared_ptr<MemTable> Frame::get_mtable() {
  return frame->mtable;
}

Frame::SafeRet<MemUnit> Frame::get_munit(const std::size_t frame_id, const int m_index) {
  return get_frame(frame_id).get_mtable()->get_munit(m_index);
}

const Frame::RtResult& Frame::rt_result() {
  return *frame->rt_result;
}

void Frame::set_rt_result(const RtResult& result) {
  *frame->rt_result = result;
}

template<class ...P> void Frame::emplace_rt_result(RtResult::ExitCode code, P... t) {
  set_rt_result(RtResult(code, t...));
}