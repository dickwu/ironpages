cmake_minimum_required(VERSION 3.15)

project(ironpages_c_package_consumer LANGUAGES C)

find_package(Ironpages ${IRONPAGES_TEST_VERSION} EXACT CONFIG REQUIRED)

add_executable(ironpages_c_package_consumer main.c)
target_compile_features(ironpages_c_package_consumer PRIVATE c_std_11)
target_link_libraries(
    ironpages_c_package_consumer
    PRIVATE "${IRONPAGES_TEST_TARGET}"
)
