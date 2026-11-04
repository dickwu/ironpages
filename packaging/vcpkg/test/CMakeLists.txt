cmake_minimum_required(VERSION 3.15)

project(ironpages_vcpkg_consumers LANGUAGES C CXX)

find_package(Ironpages 2.0.0 EXACT CONFIG REQUIRED)

add_executable(
    ironpages_c_consumer
    ../../../bindings/c/tests/package-consumers/c/main.c
)
target_compile_features(ironpages_c_consumer PRIVATE c_std_11)
target_link_libraries(ironpages_c_consumer PRIVATE Ironpages::C)

add_executable(
    ironpages_cpp_consumer
    ../../../bindings/c/tests/package-consumers/cpp/main.cpp
)
target_link_libraries(ironpages_cpp_consumer PRIVATE Ironpages::CXX)
