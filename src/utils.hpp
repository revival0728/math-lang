#ifndef UTILS_HPP
#define UTILS_HPP

#include <string>
#include <functional>
#include <memory>
#include <tuple>
#include <iostream>

namespace MathLangUtils {
  namespace Debug {
    class Console {
      public:
        template<class T> friend Console& operator<<(Console& console, T val) {
          #ifdef DEBUG
            std::cerr << val;
            return console;
          #else
            return console;
          #endif
        }
    };
    static Console console;
  }

  namespace DT {
    using number_t = double;
    using number_p = std::shared_ptr<number_t>;
    using args_t = std::vector<number_p>;
    using raw_func_t = number_t(args_t&);
    using raw_func_p = number_t(*)(args_t&);
    using func_t = std::function<raw_func_t>;
    using func_p = std::shared_ptr<func_t>;
    using hash_t = int64_t;
    using exprsye_t = uint16_t;
  }

  namespace CLI {
    const std::string RT_RESULT_CODE[] = {
      "Ok",
      "Error",
      "InvalidUse",
      "UndefinedVar",
      "UndefinedFunc",
      "Null",
    };
  }

  namespace Grammer {
    const std::string ALL_OPER[] = {"=","+","*","-","/","(",")",","};
    const std::string ALL_OPER_NAMES[] = {"set", "plus", "multiply", "minus", "divide", "lparen", "rparen", "argsplit", "func", "print", "null"};
    const int OPER_RANK[]              = {0    , 2     , 3         , 2      , 3       , 5       , 5       , 1         , 4  };
    constexpr std::size_t ALL_OPER_LEN = sizeof(ALL_OPER) / sizeof(std::string);
    constexpr std::size_t OPER_RANK_LEN = sizeof(OPER_RANK) / sizeof(int);
    constexpr std::size_t ALL_OPER_NAMES_LEN = sizeof(ALL_OPER_NAMES) / sizeof(std::string);
    static_assert(ALL_OPER_LEN + 1 == OPER_RANK_LEN, "Rank count must equals to Operator count");
    static_assert(ALL_OPER_LEN + 3 == ALL_OPER_NAMES_LEN, "Operator Name count must equals to Full Operator count");
  }

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
      func,
      print,
      null,
    };
    struct Idnt {
      enum Type { Raw, Var, Func, PreValue, None } idnt_type;
      std::variant<int, MathLangUtils::DT::number_t> idnt_data;
      Idnt() : idnt_type(None) {}
      template<class T> Idnt(Type _idnt_type, const T& _idnt_data) :
        idnt_type(_idnt_type), 
        idnt_data(_idnt_data) {}
      int& idnt_id() { return std::get<0>(idnt_data); }
      MathLangUtils::DT::number_t& raw_value() { return std::get<1>(idnt_data); }
      int idnt_id_const() const { return std::get<0>(idnt_data); }
      MathLangUtils::DT::number_t raw_value_const() const { return std::get<1>(idnt_data); }
      static Idnt make_raw(MathLangUtils::DT::number_t raw_value) { return Idnt(Raw, raw_value); }
      static Idnt make_var(int idnt_id) { return Idnt(Var, idnt_id); }
      static Idnt make_func(int idnt_id) { return Idnt(Func, idnt_id); }
      static Idnt make_pre_value() { return Idnt(PreValue, -1); }
      static Idnt make_none() { return Idnt(None, -1); }
    };
    struct Instruction {
      Operator oper;
      // For Operator::func: idnts stores in reverse order
      //  e.g. [arg_2, arg_1, arg_0, func_idnt]
      std::vector<Idnt> idnts;
      Instruction() : oper(null), idnts({}) {}
      Instruction(Operator _oper, const std::vector<Idnt>& _idnts) : oper(_oper), idnts(_idnts) {}
    };

    using InstList = std::vector<Instruction>;
  }

  namespace Hash {
    DT::hash_t hash(const std::string&);
    constexpr DT::hash_t hash_cxpr(
      const char* str, 
      DT::hash_t res1 = 0, DT::hash_t res2 = 0, 
      DT::hash_t base1 = 17, DT::hash_t base2 = 61, 
      DT::hash_t M = 1e9 + 7) {
      return *str ? 
        hash_cxpr(str + 1, (res1 + (*str)) * base1 % M, (res2 + (*str)) * base2 % M, base1, base2, M) :
        res1 ^ res2;
    }
  }

  namespace String {
    #define __TO_STRING(M) #M
    #define TO_STRING(M) __TO_STRING(M)
    template<class T> inline std::string to_string(const T[]);
    template<class T> inline std::string to_string(const T&);
    template<> inline std::string to_string<std::string>(const std::string&);
    template<class T> inline std::string bs(const T&);
    template<class T, class ...P> inline std::string bs(const T&, const P&...);
    bool is_operator(const std::string&);
    bool is_number(const std::string&);
    DT::number_t to_number(const std::string&);
    void strip_self(std::string&);
    std::string strip(const std::string&);
  }

  namespace Function {
    template<std::size_t ...I> struct indices {};
    template<std::size_t N, std::size_t ...I> struct make_indices : make_indices<N - 1, N - 1, I...> {};
    template<std::size_t ...I> struct make_indices<0, I...> : indices<I...> {};

    // https://liam.page/2019/07/04/unpack-vector-as-parameters-for-functions/
    template<typename RetT, typename ...Args> struct fn_traits_def {
      static constexpr std::size_t arg_cnt = sizeof...(Args);
      using return_type = RetT;
      template<std::size_t i> struct arg {
        using type = typename std::tuple_element<i, std::tuple<Args...>>::type;
      };
    };
    template<typename T> struct fn_traits_impl;
    template<typename RetT, typename ...Args> 
    struct fn_traits_impl<RetT(Args...)> : fn_traits_def<RetT, Args...> {};
    template<typename RetT, typename ...Args>
    struct fn_traits_impl<RetT(*)(Args...)> : fn_traits_def<RetT, Args...> {};
    template<typename T> struct fn_traits : fn_traits_impl<T> {};
    // only accept function type: <number_t(number_t, number_t, ...)>
    template<typename FnType, 
             typename VecType, 
             std::size_t ...I, 
             typename Traits = fn_traits<FnType>,
             typename RetType = typename Traits::return_type>
    RetType call_func_with_indices(FnType& func, VecType& args, indices<I...>) {
      return func(*args[I]...);
    }
    template<typename FnType, 
             typename VecType, 
             typename Traits = fn_traits<FnType>,
             typename RetType = typename Traits::return_type>
    RetType call_func(FnType& func, VecType& args) {
      return call_func_with_indices(func, args, make_indices<Traits::arg_cnt>());
    }
  }
}

template<> inline std::string MathLangUtils::String::to_string<char>(const char str[]) {
  return std::string(str);
}
template <class T> inline std::string MathLangUtils::String::to_string(const T &var) {
  return std::to_string(var);
}
template<> inline std::string MathLangUtils::String::to_string<std::string>(const std::string& str) {
  return str;
}
template<class T> inline std::string MathLangUtils::String::bs(const T& var) {
  return MathLangUtils::String::to_string(var);
}
template<class T, class ...P> inline std::string MathLangUtils::String::bs(const T& var, const P&... t) {
  return MathLangUtils::String::to_string(var) += bs(t...);
}

#endif
