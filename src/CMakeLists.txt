cmake_minimum_required(VERSION 3.10)

project(MathLang VERSION 0.1.0)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)
option(DEBUG "Build with debug information" OFF)
option(PORTABLE "Build portable version" OFF)

add_executable("math-lang" main.cpp compiler.cpp runtime.cpp utils.cpp interpreter.cpp)
configure_file(config.hpp.in config.hpp)
target_include_directories("math-lang" PUBLIC ${PROJECT_BINARY_DIR})
target_compile_options("math-lang" PRIVATE -Wall)

if(DEBUG)
  target_compile_definitions("math-lang" PRIVATE DEBUG)
  target_compile_options("math-lang" PRIVATE -g)
else()
  target_compile_options("math-lang" PRIVATE -O3)
endif()

if(PORTABLE)
  target_compile_options("math-lang" PRIVATE -static)
  target_link_options("math-lang" PRIVATE -static)
endif()