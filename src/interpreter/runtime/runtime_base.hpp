#ifndef RUNTIME_BASE_HPP
#define RUNTIME_BASE_HPP

#include "utils/utils.hpp"
#include <cstddef>
#include <memory>
#include <any>

class Frame;
void executor(Frame&, const Utils::BC::Instruction&);

class Object;
template<class T> using ObjPtr = std::shared_ptr<T>;

class MemUnit {
  public:
  using Idnt = Utils::BC::Idnt;
  using number_t = Utils::DT::number_t;
  using number_p = Utils::DT::number_p;
  using func_t = Utils::DT::func_t;
  using func_p = Utils::DT::func_p;

  protected:
  ObjPtr<Object> obj_ptr;

  public:
  MemUnit();
  template<class ObjT> const ObjPtr<ObjT> get() const;
  template<class ObjT> void set(const ObjT&) noexcept;
  template<class ObjT> void set(const ObjPtr<ObjT>) noexcept;

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const RtMemUnit&);
  #endif
};

class MemTable {
  public:
  template<class T> using SafeRet = Utils::DT::SafeRet<T>;

  protected:
  MemUnit pre_value;
  std::vector<MemUnit> table;

  public:
  MemTable();
  // index == -1 -> pre_value;
  SafeRet<MemUnit&> get_munit(const int index);
  MemUnit& operator[](const int index) noexcept;
};

class Frame {
  private:
  struct __Frame;

  protected:
  using _Frame = std::shared_ptr<__Frame>;

  public:
  using RtResult = Utils::Pipline::RtResult;
  template<class T> using SafeRet = Utils::DT::SafeRet<T>;
  struct FState {
    enum State {
      exec,
      decl,
    } state;
    int decl_id;
    FState() : state(exec), decl_id(-1) {}
    FState(State _state, int _decl_id) : state(_state), decl_id(_decl_id) {}
  };

  private:
  struct __Frame {
    int frame_id;
    std::shared_ptr<MemTable> mtable;
    std::shared_ptr<RtResult> rt_result;  // All related frame shares the same rt_result
    FState fstate;
    _Frame pframe;
  };

  protected:
  _Frame frame;
  Frame(_Frame);

  public:
  Frame();
  int frame_id();
  // get frame by current frame_id
  Frame get_frame(const std::size_t id);
  Frame pframe() const;
  // set this frame to new frame, return pframe();
  Frame enter_new_frame();
  void back_to_parent();
  std::shared_ptr<MemTable> get_mtable();
  // alias for get_frame(frame_id).get_mtable()->get_munit(m_index)
  SafeRet<MemUnit> get_munit(const std::size_t frame_id, const int m_index);
  RtResult const& rt_result();
  void set_rt_result(const RtResult&);
  template<class ...P> void emplace_rt_result(RtResult::ExitCode, P...);
  FState get_state() const noexcept;
  void set_state(FState::State, int def_id);
};

class Object : std::enable_shared_from_this<Object> {
  public:
  using data_p = std::shared_ptr<std::any>;
  template<class T> using SafeRet = Utils::DT::SafeRet<T>;

  protected:
  data_p data;

  public:
  Object();
  Object(data_p);
  template<class T> std::shared_ptr<T> cast_data();
  template<class T> std::shared_ptr<T const> cast_data() const;
  template<class ObjT> ObjPtr<ObjT> cast_self() noexcept;
  template<class ObjT> ObjPtr<ObjT const> cast_self() const noexcept;
  ObjPtr<Object> to_object();
  ObjPtr<Object const> to_object() const;

  // For Object
  // Object type and its subtype cannot use method above
  // Only avaliable on ObjPtr<>
  //
  // return a new object holds by shared_ptr
  ObjPtr<Object> to_ptr() const noexcept;
  virtual bool is_valid() const { return true; };
};

class Null : public Object {
  public:
  Null() : Object() {}
  bool is_valid() const override { return data == nullptr; }
};

class Callable : public Object {
  protected:
  int _arg_cnt;

  public:
  using InstList = Utils::BC::InstList;
  using Instruction = Utils::BC::Instruction;

  Callable(int __arg_cnt);
  Callable(const InstList&);
  bool is_valid() const override;
  //  pframe->pre_value -> return value
  //  frame->mtable[0] -> arg[0]
  //  frame->mtable[1] -> arg[1]
  //  frame->mtable[2] -> arg[2]
  //  ...
  void call(Frame&) const;
  void add_inst(const Instruction&);
  int arg_cnt() const noexcept;
};

class Number : public Object {
  public:
  using number_t = Utils::DT::number_t;

  Number();
  Number(const number_t&);
  bool is_valid() const override;
  Number operator+(const Number&);
  Number operator-(const Number&);
  Number operator*(const Number&);
  Number operator/(const Number&);
};

// template function implementation
template<class ObjT> const ObjPtr<ObjT> MemUnit::get() const {
  assert(obj_ptr != nullptr);
  return obj_ptr->cast_self<ObjT>();
}

template<class ObjT> void MemUnit::set(const ObjT& value) noexcept {
  obj_ptr = value.to_ptr();
}

template<class ObjT> void MemUnit::set(const ObjPtr<ObjT> value) noexcept {
  obj_ptr = value;
}

template<class ...P> void Frame::emplace_rt_result(RtResult::ExitCode code, P... t) {
  set_rt_result(RtResult(code, t...));
}

template<class T> std::shared_ptr<T> Object::cast_data() {
  assert(data->type() == typeid(T));
  return std::shared_ptr<T>(shared_from_this(), std::any_cast<T>(data.get())); 
}

template<class T> std::shared_ptr<T const> Object::cast_data() const {
  assert(data->type() == typeid(T));
  return std::shared_ptr<T>(shared_from_this(), std::any_cast<T>(data.get())); 
}

template<class ObjT> ObjPtr<ObjT> Object::cast_self() noexcept {
  auto st = shared_from_this();
  return ObjPtr<ObjT>(st, dynamic_cast<ObjT*>(st.get()));
}

template<class ObjT> ObjPtr<ObjT const> Object::cast_self() const noexcept {
  auto st = shared_from_this();
  return ObjPtr<ObjT const>(st, dynamic_cast<ObjT *const>(st.get()));
}

#endif
