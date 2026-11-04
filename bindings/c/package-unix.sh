#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "usage: $0 <platform-architecture>" >&2
    exit 1
fi

readonly binding_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly repository_dir="$(cd "${binding_dir}/../.." && pwd)"
readonly platform="$1"
readonly version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${binding_dir}/Cargo.toml" | head -n 1)"
readonly bundle="ironpages-c-${version}-${platform}"
readonly work_dir="$(mktemp -d)"
readonly bundle_dir="${work_dir}/${bundle}"
readonly cmake_dir="${bundle_dir}/lib/cmake/Ironpages"
readonly pkg_config_dir="${bundle_dir}/lib/pkgconfig"
readonly output_dir="${repository_dir}/dist/c"
readonly cmake_command="${CMAKE:-cmake}"
trap 'rm -rf "${work_dir}"' EXIT

mkdir -p \
    "${bundle_dir}/include/ironpages" \
    "${cmake_dir}" \
    "${pkg_config_dir}" \
    "${output_dir}"
cp "${binding_dir}/include/ironpages.h" "${bundle_dir}/include/"
cp "${repository_dir}/bindings/cpp/include/ironpages.hpp" "${bundle_dir}/include/"
cp -R "${repository_dir}/bindings/cpp/include/ironpages/." \
    "${bundle_dir}/include/ironpages/"
cp "${binding_dir}/ABI.md" "${binding_dir}/README.md" "${repository_dir}/LICENSE" \
    "${bundle_dir}/"
cp "${repository_dir}/bindings/cpp/README.md" "${bundle_dir}/CPP.md"
cp "${repository_dir}/target/release/libironpages_ffi.a" "${bundle_dir}/lib/"

case "${platform}" in
    linux-*)
        cp "${repository_dir}/target/release/libironpages_ffi.so" "${bundle_dir}/lib/"
        private_libraries="-ldl -lpthread -lm"
        ;;
    macos-*)
        cp "${repository_dir}/target/release/libironpages_ffi.dylib" "${bundle_dir}/lib/"
        private_libraries="-liconv -lm"
        ;;
    *)
        echo "unsupported Unix artifact platform: ${platform}" >&2
        exit 1
        ;;
esac

"${cmake_command}" \
    -DIRONPAGES_PACKAGE_SOURCE_DIR="${binding_dir}/package/cmake" \
    -DIRONPAGES_PACKAGE_OUTPUT_DIR="${cmake_dir}" \
    -DIRONPAGES_PACKAGE_ROOT="${bundle_dir}" \
    -DIRONPAGES_VERSION="${version}" \
    -P "${binding_dir}/package/cmake/generate-package.cmake"
sed \
    -e "s/@IRONPAGES_VERSION@/${version}/g" \
    -e "s/@IRONPAGES_PRIVATE_LIBS@/${private_libraries}/g" \
    "${binding_dir}/package/pkgconfig/ironpages.pc.in" \
    > "${pkg_config_dir}/ironpages.pc"
sed \
    -e "s/@IRONPAGES_VERSION@/${version}/g" \
    -e "s/@IRONPAGES_PRIVATE_LIBS@/${private_libraries}/g" \
    "${binding_dir}/package/pkgconfig/ironpages-static.pc.in" \
    > "${pkg_config_dir}/ironpages-static.pc"

tar -C "${work_dir}" -czf "${output_dir}/${bundle}.tar.gz" "${bundle}"
