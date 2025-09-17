cmake_minimum_required(VERSION 3.10)

project(MathLang VERSION 0.1.1)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
option(DEBUG "Build with debug information" OFF)
option(PORTABLE "Build portable version" OFF)
option(BUILD_TESTS "Build tests" ON)
include_directories(${PROJECT_BINARY_DIR}/src)

add_subdirectory(src)

if(BUILD_TESTS)
  enable_testing()
  add_subdirectory(tests)
endif()
