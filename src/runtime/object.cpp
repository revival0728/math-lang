#include "runtime_base.hpp"

Object::Object() {
  data = nullptr;
}

Object::Object(Object::data_p _data) {
  data = _data;
}

template<class T> std::shared_ptr<T> Object::cast_data() const {
  assert(data->type() == typeid(T));
  return std::shared_ptr<T>(shared_from_this(), std::any_cast<T>(data.get())); 
}

template<class ObjT> ObjPtr<ObjT> Object::cast_self() const noexcept {
  auto st = shared_from_this();
  return ObjPtr<ObjT>(st, dynamic_cast<ObjT*>(st.get()));
}

ObjPtr<Object> Object::to_object() const {
  return cast_self<Object>();
}

ObjPtr<Object> Object::to_ptr() const {
  return std::make_shared<Object>(data);
}


Callable::Callable() {
  data.reset(new std::any(InstList()));
}

Callable::Callable(const Callable::InstList& inst_list) {
  data.reset(new std::any(inst_list));
}

bool Callable::is_valid() const {
  return data->type() == typeid(InstList);
}

void Callable::call(Frame& frame) {
  auto inst_list = cast_data<InstList>();
  for(auto& inst : *inst_list) {
    ;
  }
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