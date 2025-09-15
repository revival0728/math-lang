#include "interpreter.hpp"
#include <string>

int main(int argc, const char **argv) {
  Interpreter interpreter;
  if(argc == 1) {
    interpreter.start_cli();
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
