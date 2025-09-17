#include <cassert>
#include "compiler.hpp"
#include "utils.hpp"

using namespace MathLangUtils;

// Tokenizer
Tokenizer::Tokenizer() {
  tokens = Tokenizer::Result_T();
}

bool Tokenizer::tokenized() const {
  return !tokens.empty();
}

Tokenizer::Result_Ref Tokenizer::get_tokens() {
  return tokens;
}

Tokenizer::Result_Ref Tokenizer::tokenize(const std::string& code) {
  tokens.clear();
  std::string token;
  TokenType token_type;
  for(auto& c : code) {
    if(std::isspace(c)) {
      if(!token.empty()) {
        tokens.push_back(token);
        token.clear();
      }
      continue;
    }
    if(c == '(' || c == ')') {
      if(!token.empty()) {
        tokens.push_back(token);
        token.clear();
      }
      tokens.push_back(std::string(1, c));
      continue;
    }
    if(token.empty()) {
      token.push_back(c);
      if(std::isdigit(c))
        token_type = TokenType::Num;
      else if(std::isalpha(c) || c == '_')
        token_type = TokenType::Idnt;
      else
        token_type = TokenType::Oper;
    } else {
      bool add_to_tkn;
      switch (token_type) {
      case TokenType::Num:
        add_to_tkn = std::isdigit(c) || c == '.' || c == '^' || c == 'e' || c == 'E';
        break;
      case TokenType::Idnt:
        add_to_tkn = std::isdigit(c) || std::isalpha(c);
        break;
      case TokenType::Oper:
        add_to_tkn = !std::isdigit(c) && !std::isalpha(c);
        break;
      default:
        assert(false && "Tokenizer Internal Error.");
        break;
      }
      if(add_to_tkn) {
        token.push_back(c);
      } else {
        tokens.push_back(token);
        token.clear();
        token.push_back(c);
        if(std::isdigit(c))
          token_type = TokenType::Num;
        else if(std::isalpha(c))
          token_type = TokenType::Idnt;
        else
          token_type = TokenType::Oper;
      }
    }
  }
  if(!token.empty()) tokens.push_back(token);
  return tokens;
}

#ifdef DEBUG
std::ostream& operator<<(std::ostream& os, const Tokenizer& tokenizer) {
  os << "[Tokenizer]:\n";
  os << "tokens:\n";
  for(int i = 0; i < tokenizer.tokens.size(); ++i) {
    os << i << ": [" << tokenizer.tokens[i] << "],\n";
  }
  os << "[END Tokenizer]";
  return os;
}
#endif


// Parser
Parser::Parser() {
  inst_list = InstList();
  cmpl_res = CmplStat(CmplStat::Blank, "");
  hash_oper = std::unordered_map<std::string, int>();
  for(auto& oper : Grammer::ALL_OPER) {
    hash_oper[oper] = Hash::hash(oper);
  }
  aval_idnt_id = 0;
  nidnt_table = std::unordered_map<std::string, int>();
  idnt_table = std::unordered_map<std::string, int>();
  aval_tmp_buffer_index = 0;
  tmp_buffer = std::vector<int>();
}

Parser::Result_Ref Parser::get_parse_result() {
  assert(inst_list.size() > 0 && "[math-lang Parser::get_parse_result()]: Error");
  pr_result = PrResult::make_result(this);
  return pr_result;
}

int Parser::new_idnt_id() {
  return aval_idnt_id++;
}

int Parser::get_idnt_id(const std::string& idnt) {
  auto id = idnt_table.find(idnt);
  auto nid = nidnt_table.find(idnt);
  if(id == idnt_table.end() && nid == nidnt_table.end()) {
    return nidnt_table[idnt] = new_idnt_id();
  }
  if(id != idnt_table.end()) return id->second;
  assert((id != idnt_table.end()) ^ (nid != nidnt_table.end()));
  if(nid != nidnt_table.end()) return nid->second;
  return -1;  // should never be -1
}

void Parser::merge_idnt_table() {
  idnt_table.merge(nidnt_table);
  assert(nidnt_table.empty());
}

int Parser::apply_tmp_buffer() {
  if(aval_tmp_buffer_index == tmp_buffer.size()) {
    tmp_buffer.push_back(new_idnt_id());
  }
  return tmp_buffer[aval_tmp_buffer_index++];
}

void Parser::free_tmp_buffer() {
  aval_tmp_buffer_index = 0;
}

Parser::Idnt Parser::store_pre_value() {
  Idnt temp = Idnt::make_var(apply_tmp_buffer());
  inst_list.push_back(Instruction(Operator::set, {temp, Idnt::make_pre_value()}));
  return temp;
}

std::pair<bool, Parser::Idnt> Parser::sy_algo(
  std::deque<Parser::Idnt>& idnts, 
  std::deque<Parser::Operator>& opers) {

  Debug::console << "sy_algo()\n";
  int in_func = 0;
  bool in_paren = false;
  int paren_cnt = 0;
  std::deque<Idnt> tmp_idnts, func_stk;
  std::deque<Operator> sub_opers, tmp_opers;
  while(!opers.empty() && !idnts.empty()) {
    assert((in_paren && paren_cnt > 0) || (!in_paren && paren_cnt == 0));
    Operator top_oper = opers.back(); opers.pop_back();

    // pop nested bracket tokens back to opers and idnts
    #define POP_TMP_IDNTS_TO_ORIGIN() { \
      if(!tmp_idnts.empty()) { \
        Debug::console << "pop_tmp_idnts_to_origin()\n"; \
        for(; tmp_idnts.back().idnt_type != Idnt::None; tmp_idnts.pop_back()) { \
          idnts.push_back(tmp_idnts.back()); \
          assert(!tmp_idnts.empty() && "At least contains Idnt::None"); \
        } \
        tmp_idnts.pop_back(); \
      } \
    }
    #define POP_TMP_OPERS_TO_ORIGIN() { \
      if(!tmp_opers.empty()) { \
        Debug::console << "pop_tmp_opers_to_origin()\n"; \
        for(; tmp_opers.back() != Operator::null; tmp_opers.pop_back()) { \
          opers.push_back(tmp_opers.back()); \
          assert(!tmp_opers.empty() && "At least contains Operator::null"); \
        } \
        tmp_opers.pop_back(); \
      } \
    }
    #define POP_TMP_TO_ORIGIN_DEQ() { \
      POP_TMP_IDNTS_TO_ORIGIN(); \
      POP_TMP_OPERS_TO_ORIGIN(); \
    }

    #define ADD_DIV_TO_TMP_DEQ() { \
      tmp_opers.push_back(Operator::null); \
      tmp_idnts.push_back(Idnt::make_none()); \
    }

    if(top_oper == Operator::argsplit) {
      ++in_func;
      // assert(tmp_opers.empty() && tmp_idnts.empty());
      auto res = sy_algo(idnts, sub_opers);
      assert(sub_opers.empty());
      if(!res.first) return {false, Idnt()};
      if(res.second.idnt_type == Idnt::PreValue) {
        Idnt tmp_var = Idnt::make_var(new_idnt_id());
        inst_list.push_back(Instruction(Operator::set, {tmp_var, Idnt::make_pre_value()}));
        idnts.push_back(tmp_var);
      } else {
        idnts.push_back(res.second);
      }
      continue;
    }
    if(top_oper == Operator::lparen) {
      --paren_cnt;
      if(paren_cnt == 0)  {
        in_paren = false;
      }
      auto res = sy_algo(idnts, sub_opers);
      assert(sub_opers.empty());
      if(!res.first) return {false, Idnt()};
      idnts.push_back(res.second);
      POP_TMP_TO_ORIGIN_DEQ();
      continue;
    }
    if(top_oper == Operator::rparen) {
      // Handles nested brackets
      // sperate idnts and opers from each nested brackets
      if(in_paren) {
        ADD_DIV_TO_TMP_DEQ();
      } else {
        in_paren = true;
      }
      ++paren_cnt;
      for(; !sub_opers.empty(); sub_opers.pop_back()) {
        tmp_opers.push_back(sub_opers.back());
        // same technique as handling upper rank operator below
        Idnt ti = idnts.back(); idnts.pop_back();
        if(ti.idnt_type == Idnt::PreValue)
          tmp_idnts.push_back(store_pre_value());
        else
          tmp_idnts.push_back(ti);
      }
      continue;
    }
    if(in_paren) {
      if(!in_func && top_oper == Operator::argsplit) {
        cmpl_res = CmplStat(CmplStat::Failed, "Cannot use \",\" out of function expression.");
        return {false, Idnt()};
      }
      // deque.push_front(): keeps operators in same order
      sub_opers.push_front(top_oper);
      continue;
    }
    if(top_oper == Operator::func) {
      assert(!idnts.empty());
      std::vector<Idnt> arg_list;
      for(; idnts.back().idnt_type != Idnt::Func; idnts.pop_back()) {
        arg_list.push_back(idnts.back());
      }
      Idnt func_idnt = idnts.back(); idnts.pop_back();
      if(!idnts.empty() && idnts.back().idnt_type == Idnt::PreValue) {
        idnts.pop_back();
        idnts.push_back(store_pre_value());
      }
      arg_list.push_back(func_idnt);
      inst_list.push_back(Instruction(top_oper, arg_list));
      idnts.push_back(Idnt::make_pre_value());
      in_func = 0;
      POP_TMP_TO_ORIGIN_DEQ();
      continue;
    }
    if(!in_func && !opers.empty() && !idnts.empty() && Grammer::OPER_RANK[top_oper] <= Grammer::OPER_RANK[opers.back()]) {
      ADD_DIV_TO_TMP_DEQ();
      tmp_opers.push_back(top_oper);
      // if top_idnt(ti) is PreValue, it will be covered by newest operation.
      // so it need to be saved to temp
      // e.g. a = 1 * 2 + 3 * 4
      Idnt ti = idnts.back(); idnts.pop_back();
      if(ti.idnt_type == Idnt::PreValue)
        tmp_idnts.push_back(store_pre_value());
      else
        tmp_idnts.push_back(ti);
    } else {
      Idnt f = idnts.back(); idnts.pop_back();
      Idnt s = idnts.back(); idnts.pop_back();
      inst_list.push_back(Instruction(top_oper, {s, f}));
      idnts.emplace_back(Idnt::make_pre_value());
      Debug::console << "bin oeprs\n";
      POP_TMP_TO_ORIGIN_DEQ();
    }
  }
  assert(tmp_idnts.empty() && tmp_opers.empty());
  assert(opers.empty() && idnts.size() >= 1);
  auto ret = idnts.back(); idnts.pop_back();
  return {true, ret};
}

std::pair<CmplStat, Parser::Result_Ref> Parser::parse(const Tokenizer::Result_T& tokens) {
  assert(!tokens.empty() && "tokens cannot be empty.");
  Debug::console << "parse()\n";
  inst_list.clear();

  // check is bracket valid
  {
    bool bracket_valid = true;
    int lcnt = 0, rcnt = 0;
    for(auto& tk : tokens) {
      if(tk == "(")
        ++lcnt;
      else if(tk == ")")
        ++rcnt;
      if(lcnt - rcnt < 0) {
        bracket_valid = false;
        break;
      }
    }
    if(lcnt - rcnt > 0) bracket_valid = false;
    if(!bracket_valid) {
      cmpl_res = CmplStat(CmplStat::Failed, "Invalid bracket count { left:", lcnt, ", right:", rcnt, " }");
      pr_result = PrResult::make_result(this);
      return {cmpl_res, pr_result};
    }
  }

  Debug::console << "making sy_algo stacks...\n";
  // Move and split tokens into two stacks and do sy_algo()
  std::deque<Idnt> idnts;
  std::deque<Operator> opers;

  // every bit represents expected operator or idnt
  // the order of bit equals to the reverse order of ALL_OPER
  // the last bit of it represents idnt
  DT::exprsye_t expect_bits = 0b000001101;
  assert((expect_bits | ((1 << 10) - 1)) == (1 << 10) - 1);

  #define IS_INVALID_TOKEN(rb) (((expect_bits | rb) ^ expect_bits) != 0)
  #define TOKENS_TO_OPERATOR_MAPPING(tkr, oper, rb, eb) \
  case Hash::hash_cxpr(tkr): {\
      if(IS_INVALID_TOKEN(rb)) { \
        Debug::console << "Invalid syntax detected!\n"; \
        std::string expt; \
        if(expect_bits & 1) expt.append("identifier "); \
        for(int i = 1; i < 9; ++i) { \
          if(expect_bits & (1 << i)) { \
            expt.append(Grammer::ALL_OPER[Grammer::ALL_OPER_LEN - i]); \
            expt.push_back(' '); \
          } \
        } \
        std::size_t position = tk - tokens.begin() + 1; \
        cmpl_res = CmplStat(CmplStat::Failed, "Syntax Error: expected { ", expt, "} at token position ", position, " found ", tkr); \
        return {cmpl_res, pr_result}; \
      } \
      opers.push_back(oper); \
      expect_bits = eb; \
      break; \
    }

  for(auto tk = tokens.begin(); tk != tokens.end(); ++tk) {
    switch (Hash::hash(*tk)) {
    TOKENS_TO_OPERATOR_MAPPING("=", Operator::set,      0b100000000, 0b000001001)
    TOKENS_TO_OPERATOR_MAPPING("+", Operator::plus,     0b010000000, 0b000001001)
    TOKENS_TO_OPERATOR_MAPPING("*", Operator::multiply, 0b001000000, 0b000001001)
    TOKENS_TO_OPERATOR_MAPPING("-", Operator::minus,    0b000100000, 0b000001001)
    TOKENS_TO_OPERATOR_MAPPING("/", Operator::divide,   0b000010000, 0b000001001)
    TOKENS_TO_OPERATOR_MAPPING("(", Operator::lparen,   0b000001000, 0b000001001)
    TOKENS_TO_OPERATOR_MAPPING(")", Operator::rparen,   0b000000100, 0b011110100)
    TOKENS_TO_OPERATOR_MAPPING(",", Operator::argsplit, 0b000000010, 0b000001101)
    default:
      if(IS_INVALID_TOKEN(0b000000001)) {
        Debug::console << "Invalid syntax detected!\n";
        std::string expt;
        if(expect_bits & 1) expt.append("identifier ");
        for(int i = 1; i < 9; ++i) {
          if(expect_bits & (1 << i)) {
            expt.append(Grammer::ALL_OPER[Grammer::ALL_OPER_LEN - i]);
            expt.push_back(' ');
          }
        }
        std::size_t position = tk - tokens.begin() + 1;
        cmpl_res = CmplStat(CmplStat::Failed, "Syntax Error: expected { ", expt, "} at token position ", position, " found an identifier");
        return {cmpl_res, pr_result};
      }
      if(String::is_number(*tk)) {
        idnts.push_back(Idnt::make_raw(String::to_number(*tk)));
        expect_bits = 0b011110110;
        break;
      }
      // function exists in both Idnt and Operator
      if(*std::next(tk) == "(") {
        idnts.push_back(Idnt::make_func(get_idnt_id(*tk)));
        opers.push_back(Operator::func);
        expect_bits = 0b000001000;
        break;
      }
      // Assume it is a Idnt::Var
      idnts.push_back(Idnt::make_var(get_idnt_id(*tk)));
      expect_bits = 0b111110110;
      break;
    }
  }
  Debug::console << "idnts.size(): " << idnts.size() << '\n';
  Debug::console << "oeprs.size(): " << opers.size() << '\n';
  auto [ok, res_idnt] = sy_algo(idnts, opers);
  assert(opers.empty());
  assert(idnts.empty());
  free_tmp_buffer();
  if(!ok) return {cmpl_res, pr_result};
  if(inst_list.empty()) {
    inst_list.push_back(Instruction(Operator::print, {res_idnt}));
    cmpl_res = CmplStat(CmplStat::Ok, "Compiled successfully.");
    pr_result = PrResult::make_result(this);
    merge_idnt_table();
    return {cmpl_res, pr_result};
  }
  if(inst_list.back().oper != Operator::set)
    inst_list.push_back(Instruction(Operator::print, {res_idnt}));
  cmpl_res = CmplStat(CmplStat::Ok, "Compiled successfully.");
  pr_result = PrResult::make_result(this);
  merge_idnt_table();
  return {cmpl_res, pr_result};
}

#ifdef DEBUG
std::ostream& operator<<(std::ostream& os, const Parser& p) {
  using Idnt = Parser::Idnt;

  os << "[Parser]:\n";
  os << "hash_oper:\n";
  for(auto& [k, v] : p.hash_oper)
    os << k << ": " << v << '\n';
  os << "inst_list:\n";
  for(auto& i : p.inst_list) {
    os << Grammer::ALL_OPER_NAMES[i.oper] << " [" ;
    for(auto& j : i.idnts) {
      switch(j.idnt_type) {
      case Idnt::Raw:
        os << j.raw_value_const() << ", ";
        break;
      case Idnt::None:
        os << "None" << ", ";
        break;
      case Idnt::PreValue:
        os << "PreValue" << ", ";
        break;
      case Idnt::Var:
        os << "Var(" << j.idnt_id_const() << ")" << ", ";
        break;
      case Idnt::Func:
        os << "Func(" << j.idnt_id_const() << ")" << ", ";
        break;
      }
    }
    os << "]\n";
  }
  os << "cmpl_res:\n";
  os << "[" << p.cmpl_res.code << "]: " << p.cmpl_res.msg << '\n';
  os << "idnt_table:\n";
  for(auto& [k, v] : p.idnt_table)
    os << k << ": " << v << '\n';
  os << "nidnt_table:\n";
  for(auto& [k, v] : p.nidnt_table)
    os << k << ": " << v << '\n';
  os << "aval_idnt_id: " << p.aval_idnt_id << '\n';
  os << "tmp_buffer.size(): " << p.tmp_buffer.size() << '\n';
  os << "[Parser END]";
  return os;
}
#endif

Compiler::Compiler() {}

std::pair<CmplStat, const Compiler::CmplResult&> Compiler::compile(const std::string& sline) {
  auto tokens = tokenizer.tokenize(sline);
  if(tokens.empty()) {
    Debug::console << "User input length is 0, stop at tokenization\n";
    auto cmpl_stat = CmplStat(CmplStat::Empty, "Empty line");
    auto cmpl_res = CmplResult::make_result(nullptr);
    return {cmpl_stat, cmpl_res};
  }
  auto ret = parser.parse(tokens);
  return ret;
}

#ifdef DEBUG
std::ostream& operator<<(std::ostream& os, const Compiler& c) {
  os << "[Compiler]:\n";
  os << c.tokenizer << '\n';
  os << c.parser << '\n';
  os << "[Compiler END]";
  return os;
}
#endif
