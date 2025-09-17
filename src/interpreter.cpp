#include "interpreter.hpp"
#include "utils.hpp"
#include <fstream>

using namespace Utils;

Interpreter::Interpreter() {}

std::string Interpreter::get_input(std::istream& is) {
  if(!hide_prefix) std::cout << input_prefix;
  std::string input;
  std::getline(is, input);
 String::strip_self(input);
  return input;
}

template<class ...P> void Interpreter::println(const P... t) {
  std::cout << String::bs(t...) << '\n';
}

std::pair<int, std::string> Interpreter::exec_line(const std::string& sline) {
  auto [cmpl_stat, cmpl_res] = compiler.compile(sline);
  Debug::console << compiler << '\n';
  if(cmpl_stat.code == CmplStat::Empty) {
    return {0, ""};
  }
  if(cmpl_stat.code != CmplStat::Ok) {
    return {1, cmpl_stat.msg};
  }
  auto rt_result = runtime.run(cmpl_res);
  Debug::console << runtime << '\n';
  if(rt_result.code != Runtime::RtResult::Ok) {
    return {
      2, 
      String::bs(CLI::RT_RESULT_CODE[rt_result.code], ": ", rt_result.msg)
    };
  }
  return {0, rt_result.msg};
}

void Interpreter::start_cli() {
  hide_prefix = false;
  println(PROJ_INTER_NAME, " ", PROJ_VERSION, " [", CPP_CMPL_ID, " ", CPP_CMPL_VERSION "]");
  while(1) {
    std::string sline = get_input(std::cin);
    if(sline == "quit") return;
    auto [ok, output] = exec_line(sline);
    if(!output.empty()) {
      if(ok == 1) println("Compiler Error:");
      if(ok == 2) println("Runtime Error:");
      println(output);
    }
  }
}

void Interpreter::exec_file(const std::string fn) {
  hide_prefix = true;
  std::ifstream file(fn);
  int ln = 1;
  while(!file.eof()) {
    std::string sline = get_input(file);
    auto [ok, output] = exec_line(sline);
      if(!output.empty()) {
        if(ok == 1) println("Compile Error at line ", ln, ":");
        if(ok == 2) println("Runtime Error at line ", ln, ":");
        println(output);
        if(ok > 0) return;
    }
    ++ln;
  }
  if(compiler.has_unclosed_stmt()) {
    println("Unclosed statement error: missing end keyword");
  }
}
