add_subdirectory(runtime)
add_library(Interpreter compiler.cpp interpreter.cpp)
target_include_directories(Interpreter PUBLIC ${CMAKE_SOURCE_DIR}/src)
target_link_libraries(Interpreter Runtime)