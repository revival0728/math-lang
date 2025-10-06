#ifndef UTILS_BC_HPP
#define UTILS_BC_HPP

#include "./dt.hpp"
#include <vector>
#include <variant>
#include <string>

namespace Utils {
  namespace BC {  // ByteCode
    // be aware of the rank of operators
    enum Operator {
      // actuall operator
      set,
      plus,
      multiply,
      minus,
      divide,
      lparen,
      rparen,
      argsplit,

      // virtual operator
      null,
      func,
      print,
      callbf,  // call bultin function
      def,  // define function
      ret,  // return
    };
    struct Idnt {
      enum Type { Raw, Var, Func, PreValue, Str, None } idnt_type;
      std::variant<int, Utils::DT::number_t, std::string> idnt_data;
      // frame_id is cacluated by the depth of frame
      // global frame is 0, increasing by depth
      int frame_id;
      Idnt() : idnt_type(None), frame_id(-1) {}
      template<class T> Idnt(Type _idnt_type, const T& _idnt_data, int _frame_id) :
        idnt_type(_idnt_type), 
        idnt_data(_idnt_data),
        frame_id(_frame_id) {}
      int& idnt_id() { return std::get<0>(idnt_data); }
      Utils::DT::number_t& raw_value() { return std::get<1>(idnt_data); }
      int idnt_id_const() const { return std::get<0>(idnt_data); }
      Utils::DT::number_t raw_value_const() const { return std::get<1>(idnt_data); }
      std::string idnt_str() { return std::get<2>(idnt_data); }
      std::string idnt_str_const() const { return std::get<2>(idnt_data); }
      static Idnt make_raw(Utils::DT::number_t raw_value, int frame_id) { return Idnt(Raw, raw_value, frame_id); }
      static Idnt make_var(int idnt_id, int frame_id) { return Idnt(Var, idnt_id, frame_id); }
      static Idnt make_func(int idnt_id, int frame_id) { return Idnt(Func, idnt_id, frame_id); }
      static Idnt make_pre_value(int frame_id) { return Idnt(PreValue, -1, frame_id); }
      static Idnt make_none() { return Idnt(None, -1, -1); }
      static Idnt make_str(std::string idnt_str) { return Idnt(Str, idnt_str, -1); }
    };
    struct Instruction {
      Operator oper;
      // For Operator::func: idnts stores in reverse order
      //  e.g. [arg_2, arg_1, arg_0, func_idnt]
      // For Operator::def: idnts stores in order
      //  e.g. [func_idnt, arg_0, arg_1, arg2]
      std::vector<Idnt> idnts;
      Instruction() : oper(null), idnts({}) {}
      Instruction(Operator _oper) : oper(_oper) {}
      Instruction(Operator _oper, const std::vector<Idnt>& _idnts) : oper(_oper), idnts(_idnts) {}
    };

    using InstList = std::vector<Instruction>;
  }
}

#endif