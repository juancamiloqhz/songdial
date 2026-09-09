#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
probe_runtime="${SONGDIAL_PROBE_RUNTIME:-/tmp/songdial-gst-probe-runtime}"
export GI_TYPELIB_PATH="$probe_runtime/root/usr/lib/x86_64-linux-gnu/girepository-1.0${GI_TYPELIB_PATH:+:$GI_TYPELIB_PATH}"
export LD_LIBRARY_PATH="$probe_runtime/root/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export GST_PLUGIN_PATH_1_0="$probe_runtime/plugins"
export GST_REGISTRY_1_0="$probe_runtime/registry.bin"
python3 probe.py
