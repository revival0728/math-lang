add_library(Runtime runtime.cpp frame.cpp memory.cpp object.cpp executor.cpp)
target_include_directories(Runtime PUBLIC ${CMAKE_SOURCE_DIR}/src)