#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "usage: $0 <binary> <target>" >&2
    exit 2
fi

binary=$1
target=$2
expected_name="songdial-$target"

if [[ ! -f "$binary" ]]; then
    echo "error: artifact does not exist: $binary" >&2
    exit 1
fi
if [[ ! -x "$binary" ]]; then
    echo "error: artifact is not executable: $binary" >&2
    exit 1
fi
if [[ $(basename "$binary") != "$expected_name" ]]; then
    echo "error: expected artifact name $expected_name" >&2
    exit 1
fi

binary_directory=$(cd "$(dirname "$binary")" && pwd -P)
binary="$binary_directory/$(basename "$binary")"

description=$(file -b "$binary")
case "$target" in
    aarch64-apple-darwin)
        architecture_pattern="Mach-O 64-bit executable arm64"
        ;;
    x86_64-apple-darwin)
        architecture_pattern="Mach-O 64-bit executable x86_64"
        ;;
    x86_64-unknown-linux-gnu)
        architecture_pattern="ELF 64-bit LSB.*x86-64"
        ;;
    *)
        echo "error: unsupported release target: $target" >&2
        exit 2
        ;;
esac

if ! [[ "$description" =~ $architecture_pattern ]]; then
    echo "error: artifact architecture does not match $target: $description" >&2
    exit 1
fi

package_version=$(
    sed -nE 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -n 1
)
version_output=$("$binary" --version)
if [[ "$version_output" != "songdial $package_version" ]]; then
    echo "error: unexpected version output: $version_output" >&2
    exit 1
fi

help_output=$("$binary" --help)
if [[ "$help_output" != *"Usage: songdial [OPTIONS]"* ]]; then
    echo "error: packaged binary did not print the expected help usage" >&2
    exit 1
fi
if [[ "$help_output" != *"--no-motion  Disable motion treatments"* ]]; then
    echo "error: packaged binary help omitted --no-motion" >&2
    exit 1
fi

SONGDIAL_SMOKE_BINARY="$binary" cargo test --locked --test terminal_smoke

echo "verified $binary ($description)"
