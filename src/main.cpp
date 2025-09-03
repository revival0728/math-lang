#include "interpreter.hpp"
#include <string>

int main(int argc, const char **argv) {
  Interpreter interpreter;
  if(argc == 1) {
    #ifndef DEBUG
      interpreter.start_cli();
    #else
      interpreter.start_cli();
      // interpreter.exec_line("a = ((1 * 2 + 3) * 4) + 5 * 6 + 7");
      // interpreter.exec_line("a = (1 + 2 + cos(0) - 1) / 2");
      // interpreter.exec_line("2^9");
      // interpreter.exec_line("+");
      // interpreter.exec_line("2 ^ 9");
      // interpreter.exec_line("(1 + 2 + mod(6, 8) - 1) / 2");
      // interpreter.exec_line("a = (1 + 2 + mod(6, 8) - 1) / 2");
    #endif
    return 0;
  }
  if(argc == 2) {
    std::string fn(argv[1]);
    if(fn.substr(fn.size() - 4, 4) == ".mls") {
      interpreter.exec_file(fn);
    } else {
      std::cout << "Only accpet .mls file (Math-Lang-Script)\n";
    }
    return 0;
  }
  std::cout << "Only accept 0 or 1 argument. For 0 argument, start Interpreter CLI. For 1 argument, execute the .mls file\n";
}