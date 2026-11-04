cmake_minimum_required(VERSION 3.19)

project(ironpages_package_version_consumer LANGUAGES C)

find_package(Ironpages ${IRONPAGES_TEST_VERSION_RANGE} CONFIG REQUIRED)
