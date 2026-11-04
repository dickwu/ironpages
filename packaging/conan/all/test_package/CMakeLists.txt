cmake_minimum_required(VERSION 3.15)

project(ironpages_conan_consumers LANGUAGES C CXX)

find_package(Ironpages REQUIRED CONFIG)

add_executable(ironpages_c_consumer src/c_consumer.c)
target_compile_features(ironpages_c_consumer PRIVATE c_std_11)
target_link_libraries(ironpages_c_consumer PRIVATE Ironpages::C)

add_executable(ironpages_cpp_consumer src/cpp_consumer.cpp)
target_compile_features(ironpages_cpp_consumer PRIVATE cxx_std_17)
target_link_libraries(ironpages_cpp_consumer PRIVATE Ironpages::CXX)
