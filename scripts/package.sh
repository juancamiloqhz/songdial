#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "usage: $0 <target> [output-directory]" >&2
    exit 2
fi

target=$1
output_directory=${2:-dist}

case "$target" in
    aarch64-apple-darwin | x86_64-apple-darwin | x86_64-unknown-linux-gnu) ;;
    *)
        echo "error: unsupported release target: $target" >&2
        exit 2
        ;;
esac

binary="target/$target/release/songdial"
artifact="$output_directory/songdial-$target"

cargo build --locked --release --target "$target"
mkdir -p "$output_directory"
install -m 0755 "$binary" "$artifact"

echo "$artifact"
