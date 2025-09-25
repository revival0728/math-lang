#include "runtime_base.hpp"

MemUnit::MemUnit() { 
  obj_ptr = ObjPtr<Object>();
}

template<class ObjT> const ObjPtr<ObjT> MemUnit::get() const {
  assert(obj_ptr != nullptr);
  return obj_ptr->cast_self<ObjT>();
}

template<class ObjT> void MemUnit::set(const ObjT& value) noexcept {
  obj_ptr = value.to_ptr();
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
    return os << "Func->" << (rtm.get<func_t>().second)->target<DT::raw_func_p>();
  }
  return os;
}
#endif


MemTable::MemTable() {
  table = std::vector<MemUnit>();
}

MemTable::SafeRet<MemUnit&> MemTable::get_munit(const int index) {
  static MemUnit null_unit;
  if(index >= table.size()) return SafeRet<MemUnit&>(false, null_unit);
  return SafeRet<MemUnit&>(true, this->operator[](index));
}

MemUnit& MemTable::operator[](const int index) noexcept {
  if(index == -1) return pre_value;
  return table[index];
}
