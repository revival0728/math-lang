#include "runtime_base.hpp"
#include <cassert>

Object::Object() {
  data = nullptr;
}

Object::Object(Object::data_p _data) {
  data = _data;
}

ObjPtr<Object> Object::to_object() {
  return cast_self<Object>();
}

ObjPtr<Object> Object::to_ptr() const noexcept {
  return std::make_shared<Object>(data);
}


Callable::Callable(int __arg_cnt) {
  data.reset(new std::any(InstList()));
  _arg_cnt = __arg_cnt;
}

Callable::Callable(const Callable::InstList& inst_list) {
  data.reset(new std::any(inst_list));
}

bool Callable::is_valid() const {
  return data->type() == typeid(InstList);
}

void Callable::call(Frame& frame) const {
  auto inst_list = cast_data<InstList>();
  assert(inst_list->back().oper == Utils::BC::ret);
  for(auto& inst : *inst_list) {
    executor(frame, inst);
  }
}

void Callable::add_inst(const Instruction& inst) {
  auto inst_list = cast_data<InstList>();
  inst_list->push_back(inst);
}

int Callable::arg_cnt() const noexcept {
  return _arg_cnt;
}


Number::Number() {
  data.reset(new std::any(number_t()));
}

Number::Number(const number_t& num) {
  data.reset(new std::any(num));
}

bool Number::is_valid() const {
  return data->type() == typeid(number_t);
}

Number Number::operator+(const Number& other) {
  return Number(*cast_data<number_t>() + *other.cast_data<number_t>());
}

Number Number::operator-(const Number& other) {
  return Number(*cast_data<number_t>() - *other.cast_data<number_t>());
}

Number Number::operator*(const Number& other) {
  return Number(*cast_data<number_t>() * *other.cast_data<number_t>());
}

Number Number::operator/(const Number& other) {
  return Number(*cast_data<number_t>() / *other.cast_data<number_t>());
}
