#ifndef COMPILER_HPP
#define COMPILER_HPP

#include <string>
#include <vector>
#include <iostream>
#include <utility>
#include <deque>
#include <cctype>
#include "utils.hpp"

class Tokenizer {
  public:
  using Token_T = std::string;
  using Result_T = std::vector<Token_T>;
  using Token_Ref = Token_T&;
  using Result_Ref = Result_T&;

  protected:
  Result_T tokens;
  enum TokenType { Num, Idnt, Oper };

  public:
  Tokenizer();
  bool tokenized() const;
  Result_Ref get_tokens();
  Result_Ref tokenize(const std::string&);

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const Tokenizer&);
  #endif
};

struct CmplStat {
  enum Result {
    Ok = 0,
    Failed = 1,
    Empty = 2,
    Blank = -1,
  } code;
  std::string msg;
  CmplStat() : code(Blank), msg("") {}
  CmplStat(Result _code, const char* _msg) : code(_code), msg(std::string(_msg)) {}
  CmplStat(Result _code, const std::string& _msg) : code(_code), msg(_msg) {}
  template<class ...P> CmplStat(Result _code, P... t) :
    code(_code), 
    msg(MathLangUtils::String::bs(t...)) {}
};

class Parser {
  public:
  struct CalcStep {
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
    } oper;
    struct Idnt {
      enum Type { Raw, Var, Func, PreValue, None } idnt_type;
      int idnt_id;
      MathLangUtils::DT::number_t raw_value;
      Idnt() : idnt_type(None), idnt_id(-1), raw_value(0) {}
      Idnt(Type _idnt_type, int _idnt_id, MathLangUtils::DT::number_t _raw_value) : 
        idnt_type(_idnt_type), 
        idnt_id(_idnt_id), 
        raw_value(_raw_value) {}
      static Idnt make_raw(MathLangUtils::DT::number_t raw_value) { return Idnt(Raw, -1, raw_value); }
      static Idnt make_var(int idnt_id) { return Idnt(Var, idnt_id, 0); }
      static Idnt make_func(int idnt_id) { return Idnt(Func, idnt_id, 0); }
      static Idnt make_pre_value() { return Idnt(PreValue, -1, 0); }
      static Idnt make_none() { return Idnt(None, -1, 0); }
    };
    // For Operator::func: idnts stores in reverse order
    //  e.g. [arg_2, arg_1, arg_0, func_idnt]
    std::vector<Idnt> idnts;
    CalcStep() : oper(null), idnts({}) {}
    CalcStep(Operator _oper, const std::vector<Idnt>& _idnts) : oper(_oper), idnts(_idnts) {}
  };
  struct PrResult {
    std::size_t idnt_count;
    std::vector<CalcStep> calc_list;
    std::unordered_map<std::string, int> nidnt_table;
    static PrResult make_result(const Parser *pp) {
      PrResult ret;
      if(!pp) return ret;
      ret.idnt_count = pp->aval_idnt_id;
      ret.calc_list = pp->calc_list;
      ret.nidnt_table = pp->nidnt_table;
      return ret;
    };
  } pr_result;
  using Result_T = PrResult;
  using Result_Ref = Result_T&;

  protected:
  std::unordered_map<std::string, int> hash_oper;
  CmplStat cmpl_res;
  std::vector<CalcStep> calc_list;
  std::unordered_map<std::string, int> idnt_table;
  std::unordered_map<std::string, int> nidnt_table;  // new idnt table
  int aval_idnt_id;  // avaliable idnt ID
  // get a new idnt_id
  // @return idnt_id
  int new_idnt_id();
  // get the idnt_id of std::string from new_idnt_id()
  // if does not exist, get a new one from aval_idnt_id, and add to idnt_table
  // @return idnt_id
  int get_idnt_id(const std::string& idnt); 
  // merge idnt_table and nidnt_table
  void merge_idnt_table();

  // store Idnt::PreValue to a temp memory
  //  :add CalcStep(Operator::set, {TEMP, PreValue}) to calc_list
  //  :only modifies calc_list
  // @return temp memory (Idnt)
  CalcStep::Idnt store_pre_value();  

  private:
  // Shunting Yard Algorithm
  std::pair<bool, CalcStep::Idnt> sy_algo(std::deque<CalcStep::Idnt>&, std::deque<CalcStep::Operator>&);

  public:
  Parser();
  Result_Ref get_parse_result();
  std::pair<CmplStat, Result_Ref> parse(const Tokenizer::Result_T&);

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const Parser&);
  #endif
};

class Compiler {
  public:
  using CmplResult = Parser::PrResult;

  protected:
  Tokenizer tokenizer;
  Parser parser;

  public:
  Compiler();
  std::pair<CmplStat, const CmplResult&> compile(const std::string&);
};

#endif
