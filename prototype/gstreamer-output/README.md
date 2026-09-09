# Throwaway GStreamer output experiment

Question: can independent candidate decoding preserve current audio on failure
and prevent obsolete candidates reaching one controlled output?

This is an executable integration experiment. The fixtures contain identifiable
PCM values, and the output is captured by a synchronized GStreamer `fakesink`.
It opens no audio device. It is not production code or a simulated Songdial UI.

## Run

The recorded environment is Ubuntu 26.04 x86-64, Python 3.14.4 with GI, and the
already installed GStreamer core 1.28.2. Matching extra packages were downloaded
and extracted under `/tmp`; no system packages were installed. This experiment
does not establish behavior on the researched 1.28.7 release or either macOS
architecture.

From this directory, on that environment:

```sh
bash bootstrap.sh  # requires network and matching Ubuntu package metadata
bash run.sh        # uses loopback HTTP; records results.json
```

`SONGDIAL_PROBE_RUNTIME` can select another absolute directory for extracted
dependencies. `run.sh` scopes typelib/library/plugin paths and the plugin
registry to that runtime. No dependencies are committed. The recorded package
hashes are in `packages.sha256`; these identify this experiment's inputs, not
a production release-verification policy.

## Shape

Each source decodes independently into a bounded `appsink`. Preparation caches
the first decoded data without sending it to output. A controller serializes
request generation and selection; only the selected source feeds one persistent
`appsrc` followed by the captured sink. It rebases output timestamps, limits
in-flight buffers, and waits for captured output to drain before recording
completion. The minimal session/Queue dictionary demonstrates commit ordering;
it does not exercise Songdial's Rust Application Module.

The scenario intentionally leaves one obsolete candidate decoding after another
request supersedes it. That reproduces late completion even if cancellation
does not stop a decoder immediately. Production cleanup must remain bounded;
the deliberate extra candidate here is not a lifecycle policy.

See [FINDINGS.md](FINDINGS.md) for observed results and their limits. No code
from this branch should be merged or promoted into Songdial.
