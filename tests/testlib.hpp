#include <string>
#include <initializer_list>
#include <vector>
#include <functional>
#include <iostream>
#include <exception>
#include <cstdlib>

namespace MathLangTestLib {
  class TestCaseFailed : public std::exception {
    private:
    std::string explain;

    public:
    TestCaseFailed(const std::string& runner_output, const std::string& expect_output) {
      explain.append("Expected:\n");
      explain.append(expect_output);
      explain.append("\nFound:\n");
      explain.append(runner_output);
    }
    const char* what() const noexcept {
      return std::move(explain.c_str());
    }
  };
  class TestCaseAborted : public std::exception {
    private:
    std::string explain;

    public:
    TestCaseAborted(const char* _explain) : explain(_explain) {}
    TestCaseAborted(std::string _explain) : explain(_explain) {}
    const char* what() const noexcept {
      return std::move(explain.c_str());
    }
  };
  struct TestCase {
    using input_t = std::vector<std::string>;
    input_t input;
    std::string expect_output;
    TestCase() {}
    TestCase(std::string _expect_output, std::vector<std::string> _input) : input(_input), expect_output(_expect_output) {}
    TestCase(std::initializer_list<const char*> il) {
      auto it = il.begin();
      expect_output = *it++;
      for(; it != il.end(); ++it)
        input.push_back(*it);
    }
    void run(std::function<std::string(const std::vector<std::string>&)> runner) const {
      try {
        auto output = runner(input);
        if(output == expect_output) return;
        throw TestCaseFailed(output, expect_output);
      } catch(std::exception& e) {
        std::cerr << e.what() << '\n';
        std::abort();
      }
    }
  };
}
