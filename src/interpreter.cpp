#include "interpreter.hpp"
#include "utils.hpp"

Interpreter::Interpreter() {}

std::string Interpreter::get_input(std::istream& is, bool hide_prefix = false) {
  if(!hide_prefix) std::cout << input_prefix;
  std::string input;
  std::getline(is, input);
  MathLangUtils::String::strip_self(input);
  return input;
}

template<class ...P> void Interpreter::println(const P... t) {
  std::cout << MathLangUtils::String::bs(t...) << '\n';
}

std::pair<int, std::string> Interpreter::exec_line(const std::string& sline) {
  using namespace MathLangUtils;

  auto [cmpl_stat, cmpl_res] = compiler.compile(sline);
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
      String::bs(RT_RESULT_CODE[rt_result.code], ": ", rt_result.msg)
    };
  }
  return {0, rt_result.msg};
}

void Interpreter::start_cli() {
  println(inter_name, " Version ", inter_version);
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
  std::ifstream file(fn);
  int ln = 1;
  while(!file.eof()) {
    std::string sline = get_input(file, true);
    auto [ok, output] = exec_line(sline);
      if(!output.empty()) {
        if(ok == 1) println("Compile Error at line ", ln, ":");
        if(ok == 2) println("Runtime Error at line ", ln, ":");
        println(output);
        if(ok > 0) return;
    }
    ++ln;
  }
}