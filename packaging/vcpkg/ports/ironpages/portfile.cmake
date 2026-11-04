vcpkg_from_github(
    OUT_SOURCE_PATH SOURCE_PATH
    REPO dickwu/ironpages
    REF "v${VERSION}"
    SHA512 0 # TBD after the v2.0.0 tag
    HEAD_REF main
)

vcpkg_download_distfile(
    CARGO_LOCK
    URLS "https://raw.githubusercontent.com/dickwu/ironpages/v${VERSION}/Cargo.lock"
    FILENAME "ironpages-${VERSION}-Cargo.lock"
    SHA512 0 # TBD after the v2.0.0 tag
)
configure_file("${CARGO_LOCK}" "${SOURCE_PATH}/Cargo.lock" COPYONLY)

find_program(
    CARGO
    NAMES cargo cargo.exe
    HINTS
        "$ENV{CARGO_HOME}/bin"
        "$ENV{USERPROFILE}/.cargo/bin"
        "$ENV{HOME}/.cargo/bin"
    REQUIRED
)

string(REGEX MATCH "^[^-]+" HOST_ARCHITECTURE "${HOST_TRIPLET}")
if(NOT HOST_ARCHITECTURE STREQUAL VCPKG_TARGET_ARCHITECTURE)
    message(FATAL_ERROR "Ironpages requires a native Rust toolchain for ${TARGET_TRIPLET}.")
endif()
if(VCPKG_TARGET_IS_WINDOWS AND NOT VCPKG_HOST_IS_WINDOWS)
    message(FATAL_ERROR "Ironpages cannot cross-compile from ${HOST_TRIPLET} to ${TARGET_TRIPLET}.")
elseif(VCPKG_TARGET_IS_OSX AND NOT VCPKG_HOST_IS_OSX)
    message(FATAL_ERROR "Ironpages cannot cross-compile from ${HOST_TRIPLET} to ${TARGET_TRIPLET}.")
elseif(VCPKG_TARGET_IS_LINUX AND NOT VCPKG_HOST_IS_LINUX)
    message(FATAL_ERROR "Ironpages cannot cross-compile from ${HOST_TRIPLET} to ${TARGET_TRIPLET}.")
endif()

function(ironpages_build profile cargo_profile)
    set(target_dir "${CURRENT_BUILDTREES_DIR}/${profile}")
    set(profile_argument)
    if(cargo_profile STREQUAL "release")
        set(profile_argument --release)
    endif()
    vcpkg_execute_required_process(
        COMMAND
            "${CARGO}" build
            --locked
            --manifest-path "${SOURCE_PATH}/Cargo.toml"
            --package ironpages-ffi
            --target-dir "${target_dir}"
            ${profile_argument}
        WORKING_DIRECTORY "${SOURCE_PATH}"
        LOGNAME "cargo-build-${profile}"
    )
endfunction()

if(NOT DEFINED VCPKG_BUILD_TYPE OR VCPKG_BUILD_TYPE STREQUAL "release")
    ironpages_build(release release)
endif()
if(NOT DEFINED VCPKG_BUILD_TYPE OR VCPKG_BUILD_TYPE STREQUAL "debug")
    ironpages_build(debug debug)
endif()

file(INSTALL "${SOURCE_PATH}/bindings/c/include/ironpages.h"
    DESTINATION "${CURRENT_PACKAGES_DIR}/include")
file(INSTALL "${SOURCE_PATH}/bindings/cpp/include/ironpages.hpp"
    DESTINATION "${CURRENT_PACKAGES_DIR}/include")
file(INSTALL "${SOURCE_PATH}/bindings/cpp/include/ironpages"
    DESTINATION "${CURRENT_PACKAGES_DIR}/include")

function(ironpages_install_library profile destination)
    set(source_dir "${CURRENT_BUILDTREES_DIR}/${profile}/${profile}")
    if(VCPKG_LIBRARY_LINKAGE STREQUAL "dynamic")
        if(VCPKG_TARGET_IS_WINDOWS)
            file(INSTALL "${source_dir}/ironpages_ffi.dll"
                DESTINATION "${CURRENT_PACKAGES_DIR}/${destination}bin")
            file(INSTALL "${source_dir}/ironpages_ffi.dll.lib"
                DESTINATION "${CURRENT_PACKAGES_DIR}/${destination}lib")
        elseif(VCPKG_TARGET_IS_OSX)
            file(INSTALL "${source_dir}/libironpages_ffi.dylib"
                DESTINATION "${CURRENT_PACKAGES_DIR}/${destination}lib")
        else()
            file(INSTALL "${source_dir}/libironpages_ffi.so"
                DESTINATION "${CURRENT_PACKAGES_DIR}/${destination}lib")
        endif()
    elseif(VCPKG_TARGET_IS_WINDOWS)
        file(INSTALL "${source_dir}/ironpages_ffi.lib"
            DESTINATION "${CURRENT_PACKAGES_DIR}/${destination}lib")
    else()
        file(INSTALL "${source_dir}/libironpages_ffi.a"
            DESTINATION "${CURRENT_PACKAGES_DIR}/${destination}lib")
    endif()
endfunction()

if(NOT DEFINED VCPKG_BUILD_TYPE OR VCPKG_BUILD_TYPE STREQUAL "release")
    ironpages_install_library(release "")
endif()
if(NOT DEFINED VCPKG_BUILD_TYPE OR VCPKG_BUILD_TYPE STREQUAL "debug")
    ironpages_install_library(debug "debug/")
endif()

if(VCPKG_LIBRARY_LINKAGE STREQUAL "dynamic")
    set(IRONPAGES_LIBRARY_TYPE SHARED)
else()
    set(IRONPAGES_LIBRARY_TYPE STATIC)
endif()
configure_file(
    "${CMAKE_CURRENT_LIST_DIR}/IronpagesConfig.cmake.in"
    "${CURRENT_PACKAGES_DIR}/share/ironpages/IronpagesConfig.cmake"
    @ONLY
)
include(CMakePackageConfigHelpers)
write_basic_package_version_file(
    "${CURRENT_PACKAGES_DIR}/share/ironpages/IronpagesConfigVersion.cmake"
    VERSION "${VERSION}"
    COMPATIBILITY SameMajorVersion
)
file(INSTALL "${CMAKE_CURRENT_LIST_DIR}/usage"
    DESTINATION "${CURRENT_PACKAGES_DIR}/share/ironpages")
vcpkg_install_copyright(FILE_LIST "${SOURCE_PATH}/LICENSE")
