#ifndef COMPILER_HPP
#define COMPILER_HPP

#include <string>
#include <vector>
#include <iostream>
#include <utility>
#include <deque>
#include <cctype>
#include <memory>
#include <forward_list>
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
    msg(Utils::String::bs(t...)) {}
};

class FIP {  // Frame In Parser
  protected:
  struct __FIP;
  using _FIP = std::shared_ptr<__FIP>;

  struct __FIP {
    int frame_id;
    std::unordered_map<std::string, int> idnt_table;
    std::vector<int> tmp_buffer;  // temporary buffer during calculating
    int aval_tmp_buffer_index;  // avaliable temporary idnt
    int aval_idnt_id;  // avaliable idnt ID
    _FIP pframe;
  };
  _FIP fptr;

  public:
  FIP();
  FIP pframe() const;
  // return pframe()
  FIP enter_new_frame();
  void back_to_parent();
  // get a new idnt_id
  // @return idnt_id
  int new_idnt_id();
  // get the idnt_id of std::string from new_idnt_id()
  // if does not exist, get a new one from aval_idnt_id, and add to idnt_table
  // @return idnt_id
  int get_idnt_id(const std::string& idnt); 
  // get avaliable temp buffer from tmp_buffer, if not push new element to tmp_buffer and return it
  int apply_tmp_buffer();
  // set aval_tmp_buffer_index to 0
  void free_tmp_buffer();
  int idnt_count() const;
  int frame_id() const;

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const FIP&);
  #endif
};

class Parser {
  public:
  using Result_T = Utils::Pipline::PrResult;
  using Result_Ref = Result_T&;

  using Instruction = Utils::BC::Instruction;
  using InstList = Utils::BC::InstList;
  using Operator = Utils::BC::Operator;
  using Idnt = Utils::BC::Idnt;

  protected:
  std::unordered_map<std::string, int> hash_oper;
  std::unordered_map<std::string, int> hash_kw;  // hash_keyword
  CmplStat cmpl_stat;
  Utils::Pipline::PrResult pr_result;
  InstList inst_list;
  FIP frame;

  // store Idnt::PreValue to a temp memory
  //  :add CalcStep(Operator::set, {TEMP, PreValue}) to inst_list
  //  :only modifies inst_list
  // @return temp buffer (Idnt)
  Idnt store_pre_value(FIP&);
  Result_T make_result();

  private:
  // Shunting Yard Algorithm
  std::pair<bool, Idnt> sy_algo(std::deque<Idnt>&, std::deque<Operator>&);
  using ParseRange = std::pair<Tokenizer::Result_T::const_iterator, Tokenizer::Result_T::const_iterator>;
  bool is_bracket_valid(ParseRange);

  protected:
  CmplStat parse_expr(ParseRange);
  CmplStat parse_stmt(ParseRange);

  public:
  Parser();
  int frame_depth();
  Result_Ref get_parse_result();
  std::pair<CmplStat, Result_Ref> parse(const Tokenizer::Result_T&);

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const Parser&);
  #endif
};

class Compiler {
  public:
  using CmplResult = Utils::Pipline::PrResult;

  protected:
  Tokenizer tokenizer;
  Parser parser;

  public:
  Compiler();
  bool has_unclosed_stmt();
  std::pair<CmplStat, const CmplResult&> compile(const std::string&);

  #ifdef DEBUG
    friend std::ostream& operator<<(std::ostream&, const Compiler&);
  #endif
};

#endif
