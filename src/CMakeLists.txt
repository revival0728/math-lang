add_subdirectory(runtime)

configure_file(config.hpp.in config.hpp)

add_library(MathLangInterpreter compiler.cpp utils.cpp interpreter.cpp)
target_link_libraries(MathLangInterpreter Runtime)

add_executable("math-lang" main.cpp)
target_link_libraries("math-lang" MathLangInterpreter)
target_compile_options("math-lang" PRIVATE -Wall)

if(DEBUG)
  target_compile_definitions(MathLangInterpreter PRIVATE DEBUG)
  target_compile_options(MathLangInterpreter PRIVATE -g)
  target_compile_definitions("math-lang" PRIVATE DEBUG)
  target_compile_options("math-lang" PRIVATE -g)
else()
  target_compile_options(MathLangInterpreter PRIVATE -O3)
  target_compile_options("math-lang" PRIVATE -O3)
endif()

if(PORTABLE)
  target_compile_options("math-lang" PRIVATE -static)
  target_link_options("math-lang" PRIVATE -static)
endif()
