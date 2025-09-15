#include <cassert>
#include "testlib.hpp"
#include "../src/interpreter.hpp"

using namespace MathLangTestLib;

const TestCase tests[] = {
  {"57.000000", "a = ((1 * 2 + 3) * 4) + 5 * 6 + 7", "a"},
  {"1.500000", "a = (1 + 2 + cos(0) - 1) / 2", "a"},
  {"512.000000", "2^9"},
  {"4.000000", "(1 + 2 + mod(6, 8) - 1) / 2"},
  {"4.000000", "a = (1 + 2 + mod(6, 8) - 1) / 2", "a"},
  {"60.000000", "a = 7", 
                "b = 7", "c = 7", 
                "cosRad = (pow(a, 2) + pow(b, 2) - pow(c, 2)) / (2 * a * b)",
                "deg = acos(cosRad) / pi * 180",
                "deg"}
};

int main() {
  for(auto& test : tests) {
    test.run([](const TestCase::input_t& input) -> std::string {
      Interpreter inter;
      std::string output;
      for(auto& i : input) {
        auto [code, out] = inter.exec_line(i);
        if(code != 0) throw TestCaseAborted("Interpreter returned non-zero code.");
        output.append(out);
      }
      return output;
    });
  }
}
