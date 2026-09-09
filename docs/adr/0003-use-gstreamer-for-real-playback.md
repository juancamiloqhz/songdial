# Use GStreamer for the first real playback adapter

Use GStreamer with adapter-controlled candidate preparation and output selection
to preserve the existing Playback session on preparation failure and prevent a
superseded pending request from becoming audible. Ordinary CLIamp and mpv
replacement commands do not uphold that contract; the [alternative research](../research/real-playback-adapter-alternatives.md)
and [Linux prototype](../handoffs/prototype-gstreamer-output.md) support accepting
the additional buffer and lifecycle ownership this approach requires. The
prototype code is not production code, and audible output, codec support, and
installed builds on Linux x86-64 and both macOS architectures remain required
implementation checks.

Downloaded packages include the required native runtime and plugins, preserving
the intended installation experience without a separate GStreamer setup step.
