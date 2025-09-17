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
  struct PrResult {
    std::size_t idnt_count;
    MathLangUtils::BC::InstList inst_list;
    std::unordered_map<std::string, int> nidnt_table;
    static PrResult make_result(const Parser *pp) {
      PrResult ret;
      if(!pp) return ret;
      ret.idnt_count = pp->aval_idnt_id;
      ret.inst_list = pp->inst_list;
      ret.nidnt_table = pp->nidnt_table;
      return ret;
    };
  } pr_result;
  using Result_T = PrResult;
  using Result_Ref = Result_T&;

  using Instruction = MathLangUtils::BC::Instruction;
  using InstList = MathLangUtils::BC::InstList;
  using Operator = MathLangUtils::BC::Operator;
  using Idnt = MathLangUtils::BC::Idnt;

  protected:
  std::unordered_map<std::string, int> hash_oper;
  CmplStat cmpl_res;
  InstList inst_list;
  std::unordered_map<std::string, int> idnt_table;
  std::unordered_map<std::string, int> nidnt_table;  // new idnt table
  std::vector<int> tmp_buffer;  // temporary buffer during calculating
  int aval_tmp_buffer_index;  // avaliable temporary idnt
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
  // get avaliable temp buffer from tmp_buffer, if not push new element to tmp_buffer and return it
  int apply_tmp_buffer();
  // set aval_tmp_buffer_index to 0
  void free_tmp_buffer();

  // store Idnt::PreValue to a temp memory
  //  :add CalcStep(Operator::set, {TEMP, PreValue}) to inst_list
  //  :only modifies inst_list
  // @return temp buffer (Idnt)
  Idnt store_pre_value();  

  private:
  // Shunting Yard Algorithm
  std::pair<bool, Idnt> sy_algo(std::deque<Idnt>&, std::deque<Operator>&);

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

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const Compiler&);
  #endif
};

#endif
