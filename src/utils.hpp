#ifndef UTILS_HPP
#define UTILS_HPP

#include <string>
#include <functional>
#include <memory>
#include <tuple>
#include <initializer_list>

namespace Utils {
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
    // every bit represents expected operator or idnt
    // the order of bit equals to the reverse order of ALL_OPER
    // the last bit of it represents idnt
    using exprsybit_t = uint16_t;
    template<class T> using SafeRet = std::pair<bool, T>;
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
    const std::string ALL_KEYWORD[] = {"func", "end"};
    const std::string ALL_OPER[] = {"=","+","*","-","/","(",")",","};
    const std::string ALL_OPER_NAMES[] = {"set"      , "plus"     , "multiply" , "minus"    ,"divide"    , "lparen"   , "rparen"   , "argsplit"  , "null", "func", "print", "callbf", "def", "ret"};
    const int OPER_RANK[]              = {0          , 2          , 3          , 2          , 3          , 5          , 5          , 1           , 0     , 4  };
    const DT::exprsybit_t OPER_BIT[]   = {0b100000000, 0b010000000, 0b001000000, 0b000100000, 0b000010000, 0b000001000, 0b000000100, 0b000000010 , 0b000000001};
    constexpr std::size_t ALL_OPER_LEN = sizeof(ALL_OPER) / sizeof(std::string);
    constexpr std::size_t OPER_RANK_LEN = sizeof(OPER_RANK) / sizeof(int);
    constexpr std::size_t ALL_OPER_NAMES_LEN = sizeof(ALL_OPER_NAMES) / sizeof(std::string);
    constexpr std::size_t OPER_BIT_LEN = sizeof(OPER_BIT) / sizeof(DT::exprsybit_t);
    static_assert(ALL_OPER_LEN + 2 == OPER_RANK_LEN, "Rank count must equals to Operator count");
    static_assert(ALL_OPER_LEN + 6 == ALL_OPER_NAMES_LEN, "Operator Name count must equals to Full Operator count");
    static_assert(OPER_BIT_LEN + 5 == ALL_OPER_NAMES_LEN, "Operator Name count must equals to Full Operator count");
    bool is_invalid(DT::exprsybit_t, DT::exprsybit_t); 
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
      null,
      func,
      print,
      callbf,  // call bultin function
      def,  // define function
      fheader, // function header
      ret,  // return
    };
    struct Idnt {
      enum Type { Raw, Var, Func, PreValue, None } idnt_type;
      std::variant<int, Utils::DT::number_t> idnt_data;
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
      static Idnt make_raw(Utils::DT::number_t raw_value, int frame_id) { return Idnt(Raw, raw_value, frame_id); }
      static Idnt make_var(int idnt_id, int frame_id) { return Idnt(Var, idnt_id, frame_id); }
      static Idnt make_func(int idnt_id, int frame_id) { return Idnt(Func, idnt_id, frame_id); }
      static Idnt make_pre_value(int frame_id) { return Idnt(PreValue, -1, frame_id); }
      static Idnt make_none() { return Idnt(None, -1, -1); }
    };
    struct Instruction {
      Operator oper;
      // For Operator::func: idnts stores in reverse order
      //  e.g. [arg_2, arg_1, arg_0, func_idnt]
      std::vector<Idnt> idnts;
      Instruction() : oper(null), idnts({}) {}
      Instruction(Operator _oper) : oper(_oper) {}
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
    template<> inline std::string to_string<DT::exprsybit_t>(const DT::exprsybit_t&);
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

  namespace Pipline {
    struct PrResult {
      Utils::BC::InstList inst_list;
    };
    using CmplResult = PrResult;

    struct RtResult {
      enum ExitCode {
        Ok,
        Error,
        InvalidUse,
        UndefinedVar,
        UndefinedFunc,
        Null,
      } code;
      std::string msg;
      RtResult() {}
      RtResult(ExitCode _code, const std::string& _msg) : code(_code), msg(_msg) {}
      template<class ...P> RtResult(ExitCode _code, P... t) : 
        code(_code), 
        msg(Utils::String::bs(t...)) {}
      static RtResult make_null() { return RtResult(RtResult::Null, ""); }
      static RtResult make_corrupted_error() { return RtResult(RtResult::Error, "Corrupted bytecode instruction"); }
      static RtResult make_unknown_error() { return RtResult(RtResult::Error, "Unknown Error"); }
    } rt_result;
  }
}

template<> inline std::string Utils::String::to_string<char>(const char str[]) {
  return std::string(str);
}
template <class T> inline std::string Utils::String::to_string(const T &var) {
  return std::to_string(var);
}
template<> inline std::string Utils::String::to_string<std::string>(const std::string& str) {
  return str;
}
template<> inline std::string Utils::String::to_string<Utils::DT::exprsybit_t>(const Utils::DT::exprsybit_t& bits) {
  if(bits == 0) return "nothing";
  std::string str;
  if(bits & 1) str.append("identifier ");
  for(int i = 1; i < 9; ++i) {
    if(bits & (1 << i)) {
      str.append(Grammer::ALL_OPER[Grammer::ALL_OPER_LEN - i]);
      str.push_back(' ');
    }
  }
  return str;
}
template<class T> inline std::string Utils::String::bs(const T& var) {
  return Utils::String::to_string(var);
}
template<class T, class ...P> inline std::string Utils::String::bs(const T& var, const P&... t) {
  return Utils::String::to_string(var) += bs(t...);
}

#endif
