#!/usr/bin/env bash
# THROWAWAY: extract matching Ubuntu 26.04 packages into a private directory.
# Requires the existing GStreamer 1.28.2 core, Python GI, and Ubuntu apt metadata.
set -euo pipefail
probe_runtime="${SONGDIAL_PROBE_RUNTIME:-/tmp/songdial-gst-probe-runtime}"
mkdir -p "$probe_runtime/packages" "$probe_runtime/root" "$probe_runtime/plugins"
cd "$probe_runtime/packages"
apt-get download \
  gir1.2-gstreamer-1.0=1.28.2-1 \
  gir1.2-gst-plugins-base-1.0=1.28.2-1 \
  libgstreamer-plugins-base1.0-0=1.28.2-1 \
  gstreamer1.0-plugins-base=1.28.2-1 \
  gstreamer1.0-plugins-good=1.28.2-2ubuntu0.1 \
  liborc-0.4-0t64=1:0.4.42-2 \
  libsoup-3.0-0=3.6.6-1
for probe_package in ./*.deb; do
  dpkg-deb -x "$probe_package" "$probe_runtime/root"
done
for probe_plugin in app audioconvert audioresample audiotestsrc wavparse typefindfunctions soup; do
  ln -sf "$probe_runtime/root/usr/lib/x86_64-linux-gnu/gstreamer-1.0/libgst$probe_plugin.so" \
    "$probe_runtime/plugins/libgst$probe_plugin.so"
done
