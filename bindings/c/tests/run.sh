#!/usr/bin/env bash

set -euo pipefail

readonly test_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly repository_dir="$(cd "${test_dir}/../../.." && pwd)"
readonly profile="${IRONPAGES_PROFILE:-debug}"
readonly library_dir="${repository_dir}/target/${profile}"
readonly work_dir="$(mktemp -d)"
readonly host="$(uname -s)"
trap 'rm -rf "${work_dir}"' EXIT

cd "${repository_dir}"
build_arguments=(build -p ironpages-ffi)
if [[ "${profile}" == "release" ]]; then
    build_arguments+=(--release)
elif [[ "${profile}" != "debug" ]]; then
    echo "unsupported Rust build profile: ${profile}" >&2
    exit 1
fi
cargo "${build_arguments[@]}"

compile_static_smoke() {
    "${CC:-cc}" \
        -std=c11 \
        -g \
        -Wall \
        -Wextra \
        -Werror \
        -I bindings/c/include \
        bindings/c/tests/smoke.c \
        "${library_dir}/libironpages_ffi.a" \
        "$@" \
        -o "${work_dir}/ironpages-c-static-smoke"
}

if [[ "${host}" == "Linux" ]]; then
    compile_static_smoke -ldl -lpthread -lm
else
    compile_static_smoke
fi

"${CC:-cc}" \
    -std=c11 \
    -g \
    -Wall \
    -Wextra \
    -Werror \
    -I bindings/c/include \
    bindings/c/tests/smoke.c \
    -L "${library_dir}" \
    -lironpages_ffi \
    -o "${work_dir}/ironpages-c-smoke"

test_command=(
    "${work_dir}/ironpages-c-smoke"
    tests/parity/fonts/ParitySans.ttf
    tests/fonts/NotoEmoji-TestSubset.ttf
)
static_test_command=(
    "${work_dir}/ironpages-c-static-smoke"
    tests/parity/fonts/ParitySans.ttf
    tests/fonts/NotoEmoji-TestSubset.ttf
)

"${static_test_command[@]}"

case "${host}" in
    Darwin)
        DYLD_LIBRARY_PATH="${library_dir}" "${test_command[@]}"
        ;;
    Linux)
        if [[ "${IRONPAGES_VALGRIND:-0}" == "1" ]]; then
            command -v valgrind >/dev/null
            LD_LIBRARY_PATH="${library_dir}" valgrind \
                --leak-check=full \
                --track-origins=yes \
                --show-leak-kinds=definite \
                --errors-for-leak-kinds=definite \
                --error-exitcode=1 \
                "${test_command[@]}"
        else
            LD_LIBRARY_PATH="${library_dir}" "${test_command[@]}"
        fi
        ;;
    *)
        echo "unsupported C smoke-test host: ${host}" >&2
        exit 1
        ;;
esac
