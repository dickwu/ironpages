cmake_minimum_required(VERSION 3.19)

foreach(
    required_variable
    IRONPAGES_PACKAGE_SOURCE_DIR
    IRONPAGES_PACKAGE_OUTPUT_DIR
    IRONPAGES_PACKAGE_ROOT
    IRONPAGES_VERSION
)
    if(NOT DEFINED ${required_variable})
        message(FATAL_ERROR "${required_variable} is required")
    endif()
endforeach()

include(CMakePackageConfigHelpers)

set(IRONPAGES_INCLUDE_INSTALL_DIR include)
set(IRONPAGES_LIBRARY_INSTALL_DIR lib)

configure_package_config_file(
    "${IRONPAGES_PACKAGE_SOURCE_DIR}/IronpagesConfig.cmake.in"
    "${IRONPAGES_PACKAGE_OUTPUT_DIR}/IronpagesConfig.cmake"
    INSTALL_DESTINATION lib/cmake/Ironpages
    INSTALL_PREFIX "${IRONPAGES_PACKAGE_ROOT}"
    PATH_VARS
        IRONPAGES_INCLUDE_INSTALL_DIR
        IRONPAGES_LIBRARY_INSTALL_DIR
)
write_basic_package_version_file(
    "${IRONPAGES_PACKAGE_OUTPUT_DIR}/IronpagesConfigVersion.cmake"
    VERSION "${IRONPAGES_VERSION}"
    COMPATIBILITY SameMajorVersion
)
