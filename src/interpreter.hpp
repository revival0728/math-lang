#ifndef INTERPRETER_HPP
#define INTERPRETER_HPP

#include "compiler.hpp"
#include "runtime.hpp"
#include "config.hpp"
#include <string>
#include <utility>
#include <iostream>
#include <fstream>

#define PROJ_INTER_NAME TO_STRING(PROJ_NAME)
#define PROJ_VERSION TO_STRING(PROJ_VERSION_MAJOR.PROJ_VERSION_MINOR.PROJ_VERSION_PATCH)
#define CPP_CMPL_ID TO_STRING(PROJ_CPP_CMPL_ID)
#define CPP_CMPL_VERSION TO_STRING(PROJ_CPP_CMPL_VERSION)

class Interpreter {
  private:
  const char* input_prefix = ">> ";

  protected:
  Compiler compiler;
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