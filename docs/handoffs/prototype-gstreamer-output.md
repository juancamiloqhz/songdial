# Handoff: GStreamer controlled-output prototype

## Question and result

Can independent GStreamer candidate decoding preserve current audio through
preparation failures and prevent obsolete candidates reaching one output?

The local experiment supports that composition for its exercised Linux
fixtures: 16 scenario checks passed, including candidate isolation, stale
selection rejection, preserved finite opening samples, live readiness, and
completion after captured output drains. It did not use an audio device or
establish real Station, macOS, Rust, or packaged-build behavior.

## Primary evidence

The throwaway branch is `prototype/gstreamer-output`, committed and pushed at
`bae90a022602d57e6e8afdafe8dcb44cd4961e51`. It must not be merged or promoted.

- [Findings and limits](https://github.com/juancamiloqhz/songdial/blob/bae90a022602d57e6e8afdafe8dcb44cd4961e51/prototype/gstreamer-output/FINDINGS.md)
- [Captured samples and scenario results](https://github.com/juancamiloqhz/songdial/blob/bae90a022602d57e6e8afdafe8dcb44cd4961e51/prototype/gstreamer-output/results.json)
- [Run instructions](https://github.com/juancamiloqhz/songdial/blob/bae90a022602d57e6e8afdafe8dcb44cd4961e51/prototype/gstreamer-output/README.md)

The original research and proposed probe are in
[the alternative-adapter assessment](../research/real-playback-adapter-alternatives.md).
This experiment used GStreamer 1.28.2 with privately extracted Ubuntu packages;
it did not validate the researched 1.28.7 runtime. Dependencies were not
installed into the host or committed to Git.

## Return to design

Use `grill-with-docs` with [the accepted product direction](../product.md#direction-after-the-first-milestone).
Following the interview, [ADR 0003](../adr/0003-use-gstreamer-for-real-playback.md)
selects GStreamer. The prototype itself does not approve an implementation
Interface. Station resume reconnects at the live edge; current Track failure
retains the Track and Queue with an actionable error. The user subsequently
confirmed Station retry limits, truncated-input policy, catalog selection,
playback ownership, and testing boundaries in the product brief. Continue with
`to-spec` and dependency-linked `to-tickets`. Runtime and required plugins will
be bundled; metadata and attribution ship locally while finite audio loads on
demand. See [catalog research](../research/initial-real-catalog.md) for candidates.

The prototype shows why source EOS and output completion must be distinguished,
why an ERROR must not be erased by a subsequent EOS, and why remote EOF alone
does not prove a Track was complete. Carry these observations into the design;
do not inherit the prototype's minimal session dictionary as production policy.

Draft PR #22 and issue #11 remain deferred and untouched. There are no participant
observations or usability conclusions from this experiment.
