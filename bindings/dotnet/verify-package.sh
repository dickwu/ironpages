#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <package.nupkg> <version>" >&2
  exit 2
fi

PACKAGE="$1"
VERSION="$2"

if [[ ! -f "$PACKAGE" ]]; then
  echo "NuGet package not found: $PACKAGE" >&2
  exit 1
fi

REQUIRED_ENTRIES=(
  "Ironpages.nuspec"
  "LICENSE"
  "README.md"
  "lib/net8.0/Ironpages.dll"
  "runtimes/linux-x64/native/libironpages_ffi.so"
  "runtimes/linux-arm64/native/libironpages_ffi.so"
  "runtimes/osx-x64/native/libironpages_ffi.dylib"
  "runtimes/osx-arm64/native/libironpages_ffi.dylib"
  "runtimes/win-x64/native/ironpages_ffi.dll"
)

PACKAGE_ENTRIES="$(unzip -Z1 "$PACKAGE")"
for entry in "${REQUIRED_ENTRIES[@]}"; do
  if ! grep -Fqx "$entry" <<<"$PACKAGE_ENTRIES"; then
    echo "NuGet package is missing $entry" >&2
    exit 1
  fi
done

NATIVE_COUNT="$(grep -c '^runtimes/.*/native/' <<<"$PACKAGE_ENTRIES")"
if [[ "$NATIVE_COUNT" -ne 5 ]]; then
  echo "NuGet package must contain exactly five native assets; found $NATIVE_COUNT" >&2
  exit 1
fi

if ! unzip -p "$PACKAGE" Ironpages.nuspec |
  grep -Fq "<version>$VERSION</version>"; then
  echo "NuGet manifest version does not match $VERSION" >&2
  exit 1
fi

echo "NuGet package contract passed for Ironpages $VERSION."
