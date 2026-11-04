cmake_minimum_required(VERSION 3.15)

project(ironpages_cpp_package_consumer LANGUAGES CXX)

find_package(Ironpages ${IRONPAGES_TEST_VERSION} EXACT CONFIG REQUIRED)

add_executable(ironpages_cpp_package_consumer main.cpp)
target_link_libraries(
    ironpages_cpp_package_consumer
    PRIVATE "${IRONPAGES_TEST_TARGET}"
)
