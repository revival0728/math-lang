#ifndef INTERPRETER_HPP
#define INTERPRETER_HPP

#include "compiler.hpp"
#include "runtime.hpp"
#include <string>
#include <utility>
#include <iostream>
#include <fstream>

class Interpreter {
  private:
  std::string input_prefix = ">> ";
  std::string inter_name = "Math-Lang-Script Interpreter";
  std::string inter_version = "0.1.0";

  protected:
  Tokenizer tokenizer;
  Parser parser;
  Runtime runtime;
  std::string get_input(std::istream&, bool);
  template<class ...P> void println(const P...);

  public:
  Interpreter();
  // @return { code, output }
  // code = { 0 -> Ok, 1 -> Compile Error, 2 -> Runtime Error }
  std::pair<int, std::string> exec_line(const std::string&);
  void start_cli();
  void exec_file(const std::string);
};

#endif