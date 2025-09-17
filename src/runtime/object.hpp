#ifndef OBJECT
#define OBJECT

#include "../utils.hpp"
#include <utility>
#include <memory>
#include <any>

class Object {
  public:
  template<class T> using SafeRetT = std::pair<bool, T>;
  using data_t = std::shared_ptr<std::any>;

  protected:
  data_t data;

  public:
  virtual SafeRetT<data_t> get() { return {false, nullptr}; }
  virtual SafeRetT<Object> call() { return {false, Object()}; }
};

class FuncObj : public Object {
  public:
  using InstList = Utils::BC::InstList;

  private:
  InstList& __get();

  public:
  FuncObj();
  FuncObj(const InstList&);
  SafeRetT<Object> call() override;
};

#endif
