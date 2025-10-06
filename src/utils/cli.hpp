#ifndef UTILS_CLI_HPP
#define UTILS_CLI_HPP

#include <string>

namespace Utils {
  namespace CLI {
    const std::string RT_RESULT_CODE[] = {
      "Ok",
      "Error",
      "InvalidUse",
      "UndefinedVar",
      "UndefinedFunc",
      "Null",
    };
  }
}

#endif