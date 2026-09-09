# Real playback alternatives: mpv and GStreamer

Research date: 2026-09-09. Status: primary-source assessment and proposed prototype scope. Neither backend was installed, built, launched, or listened to during this research.

Subsequent evidence: the [controlled-output prototype handoff](../handoffs/prototype-gstreamer-output.md) records a separate Linux/GStreamer 1.28.2 experiment, its immutable findings, and its limits. The user then selected GStreamer for the first implementation in [ADR 0003](../adr/0003-use-gstreamer-for-real-playback.md). The research below describes the source evidence and recommendation that preceded that decision.

## Decision context and recommendation

The Listener-facing guarantees remain: current audio continues while a candidate loads; failed preparation preserves the current Playback session and Queue; only the newest pending request may become current; success updates current item and Queue together. The user chose to preserve these guarantees, research alternatives to CLIamp, and begin with curated public Stations plus a small real Track/Playlist catalog without accounts. These are accepted product constraints, not findings about either backend. [Product brief](../product.md#direction-after-the-first-milestone), [interaction model](../interaction-model.md#playback-and-queue), [CLIamp findings](cliamp-playback-adapter.md)

**Recommendation:** investigate a small GStreamer adapter that controls when decoded candidate audio reaches one persistent output. It provides primitives for staging decoded data and changing the selected source, which align better with these guarantees than a stock single-player replacement command. This is a recommendation for a **bounded prototype**, not a finding that an implementation already satisfies the contract. Source isolation, live readiness, timestamps, errors, and cancellation at the output boundary remain work. [GStreamer pipeline manipulation][gst-manipulation], [input-selector source][gst-selector]

Do not select mpv merely because it has mature IPC: one mpv instance's `loadfile replace`, whether invoked through JSON IPC or libmpv, stops current playback without waiting for the candidate to succeed. A staged second mpv instance is a different, composed adapter with additional lifecycle and output coordination. [Pinned mpv command documentation][mpv-input], [command implementation][mpv-command]

The existing contract requires atomic **application state**, not sample-perfect gapless switching or a crossfade. A successful transition may have a small measurable gap. That does not excuse stopping the current session on a preparation failure or making an obsolete candidate audible. No API studied promises an atomic transaction across independent audio devices.

## Source snapshots

| Component | Evidence snapshot | Scope |
| --- | --- | --- |
| mpv | v0.41.0, published 2025-12-21; source `41f6a645068483470267271e1d09966ca3b9f413` | Latest stable release returned by upstream GitHub API during research. Pinned docs and C sources below. |
| GStreamer | 1.28.7, released 2026-09-07; source `070125524a8422e29d3b69a372ed4f62fd343ffa` | Current stable source; selected core/playback code plus first-party API/design docs. |
| gstreamer-rs | 0.25.3, source `4e20c0caa0100421080f081065e51f9341ba4693` | Official Rust bindings; workspace requires Rust 1.92, edition 2024. |

Sources: [mpv release](https://github.com/mpv-player/mpv/releases/tag/v0.41.0), [GStreamer release notes](https://gstreamer.freedesktop.org/releases/1.28/), [GStreamer tag](https://github.com/GStreamer/gstreamer/tree/1.28.7), [Rust binding manifest][gst-rust-manifest]. GStreamer's canonical development service is its linked [GitLab](https://gitlab.freedesktop.org/gstreamer/gstreamer); immutable code links use its GStreamer-organization GitHub mirror. Documentation without a version in its URL is an access-date snapshot, not a promise of unchanged future behavior.

Temporary evidence resides at `/tmp/songdial-mpv-research-v0.41.0` and `/tmp/songdial-gstreamer-research-1.28.7`. Durable claims use the links below. No exhaustive plugin or distribution survey was attempted.

## mpv: capable observation, nontransactional replacement

**Documented:** JSON IPC is local newline-delimited JSON with commands, replies, and unsolicited events. Integer `request_id` values correlate command replies; property observation requires a persistent connection. These are correlation mechanisms, not ownership of a playback generation. libmpv embeds the same playback core; `mpv_create()` defaults to no terminal/config access and an idle core. [IPC documentation][mpv-ipc], [client API][mpv-client]

**Source fact:** `cmd_loadfile()` clears the playlist for replacement, inserts the new entry, selects it, and wakes the playback core. Its result includes the new `playlist_entry_id`. It does not prepare a second source while retaining rollback ownership of the old one. [Command implementation][mpv-command]

**Readiness is layered:** `file-loaded` is emitted after playback initialization but before an optional initial cache wait and the subsequent playback loop. `on_preloaded` is earlier still: before track selection and decoder creation. `playback-restart` occurs after the core's audio/video readiness checks, but is also used after seeking. None is an application-specific, transactional replacement acknowledgement. [Load implementation][mpv-load], [hook documentation][mpv-hooks], [playback readiness source][mpv-restart]

`mpv_abort_async_command()` asks matching asynchronous commands to abort. The API explicitly says cancellation is asynchronous, command-dependent, and ineffective after the command has completed. Since `loadfile` completes as playlist manipulation, aborting its request ID is not a reliable way to suppress later audio loading. [Abort API][mpv-abort]

Gapless audio does not repair failure preservation: its documented implementation uses already-buffered audio while changing files; that buffer can run out while the next source loads. [Gapless option][mpv-gapless]

### What a composed mpv adapter would require

Proposed design, not verified behavior: keep A in one owned process/core; initialize a separate B paused with isolated configuration, audio only, and no auto-advance; reject failed B without touching A. Only an adapter controller holding the latest request generation can authorize B to leave its inaudible state. An obsolete candidate is destroyed without being enabled.

This can separate preparation failures from A, but introduces several open questions:

- What event/property combination proves B has decoded usable audio while remaining paused, including finite HTTP files and the selected Station transports?
- Can two instances prepare against the desired output without contention? mpv has explicit exclusive-output modes and an optional fallback-to-null behavior, so opening a device cannot be assumed to prove useful audible output. [Audio options][mpv-options]
- Switching A and B involves separate controls. Pausing A before B is successfully enabled risks an interruption on B's activation failure; enabling B first risks overlap. Reusing `file-loaded` does not settle that race.
- Once B's enable command has been issued, a later request cannot retroactively retract device-buffered audio. The controller must define one serialized commit boundary and distinguish already-committed playback from an older **pending** request.

Separate libmpv instances remove socket/process overhead but retain this coordination problem. Multiple client handles attached to one core are not independent candidate players. [Client creation API][mpv-client]

**Assessment:** plausible if a two-instance design and its transition policy are deliberately accepted and validated; stock one-instance mpv is a known mismatch. It is not the recommended first probe when failure preservation and explicit output selection are the deciding requirements.

## GStreamer: stage data and own the output boundary

**Documented/source facts:** a normal `playbin3` application changes the URI by taking the element back to READY/NULL and then starting it again. Its `instant-uri` option changes when URI changes apply; its `about-to-finish` signal queues a following URI. These APIs do not document Songdial's replacement rollback transaction. Simply replacing the current `playbin3` URI is not the proposed solution. [Playbin3 source][gst-playbin], [URI-switching properties][gst-playbin-switch]

For finite sources, sinks preroll by queuing a buffer before completing PAUSED. The preroll contract can also complete on GAP or EOS, so PAUSED/ASYNC_DONE alone is insufficient evidence of usable audio. GStreamer's pipeline guide demonstrates blocking decoded source pads to partially preroll before connecting/unblocking an output. [Preroll design][gst-preroll], [pipeline manipulation][gst-manipulation]

`input-selector` routes one input to an output and permits changing `active-pad` while playing. Its implementation exposes clock/active-stream synchronization, buffer caching, and dropping backwards timestamps; the documentation warns that inappropriate synchronization can lose data or introduce delays. These controls support a single-output design, but do not provide the complete adapter policy. [Selector source][gst-selector]

### Proposed adapter shape and unresolved costs

One controller owns current source A, at most one pending candidate generation B, and the decision that permits a source's decoded buffers into the output. Preparation must not mutate A or the Songdial Queue. On candidate error, timeout, or supersession, discard that candidate. Commit checks the current generation immediately at the output selection boundary, then emits one success observation used to update the Playback session and Queue together.

There are two possible implementation shapes to test; neither is approved here:

| Shape | Potential benefit | Main risk to prove |
| --- | --- | --- |
| Independently managed decode branches feeding a selector and one sink | One output device remains open; no two-device handoff. | Candidate errors/flushes must stay isolated from A; branch lifecycle, timestamp alignment, inactive flow, and EOS handling need careful ownership. |
| Independent source pipelines ending in `appsink`, with an adapter-controlled feed into one `appsrc` output pipeline | Candidate pipelines cannot directly emit audible buffers; adapter explicitly chooses which generation's buffers are forwarded. | More application-managed PCM buffering, pacing, timestamps, backpressure, and lifecycle. GStreamer recommends understanding these details rather than treating appsrc/appsink as automatic synchronization. |

`appsink` and `appsrc` are public facilities for extracting/injecting data with bounded queue controls, and injection occurs through the pipeline's streaming thread. Their existence supports feasibility, not correctness of the composed design. [Appsink API][gst-appsink], [Appsrc API][gst-appsrc], [pipeline guide][gst-manipulation]

A candidate sharing a pipeline or output path with A is **not automatically failure-isolated**. The prototype must show that candidate decode errors and teardown do not stop, flush, or stall A. Avoid a global “any ERROR stops everything” handler. Shared-output device failure is a failure of the current session as well; retaining the old source cannot make a failed device work.

Inactive-stream synchronization can advance a finite candidate before selection. A candidate also normally starts its timestamps at zero while A's output clock is already running. The adapter must preserve the candidate's intended first sample and align timestamps deliberately; otherwise synchronization or backwards-timestamp dropping can skip its opening audio. Candidate buffering must not trigger a global pipeline pause. These are composition risks to test, not guarantees supplied by the selector. [Selector synchronization source][gst-selector]

## Tracks, Stations, EOF, and pause

Songdial must retain its catalog-defined Track/Station type. GStreamer's live-source property describes scheduling, not music taxonomy: true live sources produce data only in PLAYING and return `NO_PREROLL` for PAUSED transitions. An internet Station is not guaranteed to use that exact live scheduling behavior. Therefore staging every candidate in PAUSED cannot be the sole readiness mechanism. A live candidate may need to run into an inaudible, bounded buffer path until actual decoded audio is available. [Live-source design][gst-live]

The pause product choice remains open. For Stations, decide whether resuming continues buffered audio or reconnects near the live edge. Neither generic pause nor a URL's scheme establishes that Listener-facing promise. For a first curated catalog, explicitly test the actual transport/codec combinations, including any bounded PLS/M3U resolution that yields the Station's stream endpoint. An expanded transport playlist must not become Songdial's Track Queue by accident.

**mpv observation:** `end-file` distinguishes `eof`, `stop`, `quit`, `error`, and redirects, and includes playlist-entry identity. Its own contract warns that damaged/truncated inputs or broken connections can sometimes be reported as EOF. It therefore cannot certify that every finite network Track played completely. [End-file API][mpv-end]

**GStreamer observation:** the bus distinguishes EOS from ERROR and carries state, tag, buffering, and clock messages. ERROR means the application must not assume that the affected pipeline will keep producing data. For non-live pipelines, buffering commonly requires PAUSED/PLAYING management; true live pipelines have different handling. EOS is a pipeline report, not proof of remote content integrity. [Message definitions][gst-messages]

Recommendation: Songdial owns Queue and advancement. Submit one selected source/candidate at a time, correlate observations with source instance and generation, retain the final Track as stopped, and treat Station EOS as interruption/recovery rather than natural completion. Store errors before teardown; distinguish explicit stop from EOF, candidate failure from current failure, and unknown/early remote EOF from verified full completion. Do not advance a queue from both Songdial's simulated tick and the backend.

## Linux/macOS and packaging

| Topic | mpv/libmpv | GStreamer/Rust |
| --- | --- | --- |
| Target coverage | Upstream supports modern Linux and macOS 10.15+. Its installation page links Linux packages, Homebrew/MacPorts, and source builds; most binary packages are third-party. | Upstream publishes universal macOS x86_64/ARM64 runtime/development packages and supports Linux packages/source builds. |
| Reproducible dependency | Pin executable/core and relevant FFmpeg/output configuration; libmpv adds a native library dependency. | Pin runtime and required plugin set; Rust bindings do not replace native GStreamer libraries/plugins. |
| Audio-only packaging | Avoid relying on a desktop app bundle or the Listener's player config. Output and codec support depend on the selected build. | Use explicit audio output and codec/network elements; plugin discovery must work in the installed layout. |
| Testing limitation | Null audio output can exercise protocol/timing but cannot certify device transitions or sound. | Fakesink/appsink and a test clock can exercise data selection; actual sinks still require platform testing. |

Sources: [mpv requirements][mpv-readme], [mpv installation](https://mpv.io/installation/), [GStreamer downloads](https://gstreamer.freedesktop.org/download/), [Rust binding installation][gst-rust-readme], [mpv null output][mpv-null], [GStreamer test clock][gst-testclock].

The GStreamer release page was current at 1.28.7, while the inspected download page still advertised 1.28.6 macOS packages. Do not claim a tested 1.28.7 macOS binary from that evidence. Pin the actual artifact selected for a prototype/build. The Rust README also contains old example version numbers and a historical Homebrew warning; this report does not interpret that warning as proof that today's Homebrew installation is broken. [Release notes](https://gstreamer.freedesktop.org/releases/1.28/), [downloads](https://gstreamer.freedesktop.org/download/), [binding README][gst-rust-readme]

All three existing Songdial package targets need their own installed-build verification before claiming supported real playback. Package existence is not proof of a distributable, audible Songdial build. No new distribution tooling or release work was performed here.

## Bounded prototype handoff

Recommended question: **Can a small GStreamer adapter keep A flowing while a candidate prepares or fails, and guarantee that only the current request generation can reach one output path?** The prototype should answer this before adding production adapter code, general Service abstractions, or catalog UI.

Use generated identifiable PCM signals and controlled local file/HTTP inputs. Capture the decoded output before the real sink, not merely event logs. A, B, and C should be distinguishable in samples so the result can establish which candidate actually reached output.

Acceptance probes:

1. A plays; B delays then fails decoding or returns HTTP failure. A's output continues, and A/session/Queue remain current.
2. B prepares slowly; C supersedes B; B later yields valid audio. No B samples reach output, even when readiness and cancellation race.
3. B reaches a staging readiness boundary but is canceled before selection. Teardown neither flushes A nor enables B.
4. Successful current B selection produces one application commit. Record the transition gap/overlap; sample-perfect continuity is not a requirement.
5. Repeat for finite media, an endless HTTP stream, and a true live `NO_PREROLL` source. Require actual usable decoded data, not EOS/GAP alone, before readiness.
6. Observe explicit stop, finite EOS, injected decoder error, truncated network input, and candidate/current shutdown independently. Only the chosen Queue owner advances once.
7. Hold B pending long enough to exercise bounded memory and backpressure. For finite B, verify that its intended opening samples survive staging and timestamp alignment; for live B, verify the chosen freshness policy. Candidate buffering or EOS must not pause or terminate A's output.

`GstTestClock` lets tests advance pipeline time deliberately and release clock waits; it does not control HTTP scheduling or replace wall-clock watchdogs for shutdown. Use bounded local servers and sample assertions for integration tests. Record the exact runtime and plugin versions used; the pinned research version does not imply that the host has that version installed. The production network and real hardware still require separate checks. [Test-clock source][gst-testclock]

Stop the prototype once it answers whether this shape can enforce the accepted contract and what complexity it requires. Preserve findings and return them to the design thread. If it cannot isolate candidate errors or enforce output ownership simply enough, revisit the two-instance mpv option explicitly; do not silently weaken the product guarantee.

Remaining design decisions include successful-commit readiness, Station resume/retry behavior, current Track failure/early-EOF policy, backend ownership and shutdown, chosen dependency distribution, and real catalog sources. No implementation ticket, adapter ADR, or human-validation conclusion is settled by this research.

[mpv-input]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/DOCS/man/input.rst#L543-L584
[mpv-hooks]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/DOCS/man/input.rst#L1912-L1944
[mpv-command]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/player/command.c#L6116-L6152
[mpv-ipc]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/DOCS/man/ipc.rst#L88-L142
[mpv-client]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/include/mpv/client.h#L427-L568
[mpv-load]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/player/loadfile.c#L1834-L1899
[mpv-restart]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/player/playloop.c#L1147-L1185
[mpv-abort]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/include/mpv/client.h#L1011-L1041
[mpv-options]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/DOCS/man/options.rst#L2053-L2068
[mpv-gapless]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/DOCS/man/options.rst#L2316-L2349
[mpv-end]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/include/mpv/client.h#L1460-L1530
[mpv-readme]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/README.md
[mpv-null]: https://github.com/mpv-player/mpv/blob/41f6a645068483470267271e1d09966ca3b9f413/DOCS/man/ao.rst#L245-L268
[gst-playbin]: https://github.com/GStreamer/gstreamer/blob/070125524a8422e29d3b69a372ed4f62fd343ffa/subprojects/gst-plugins-base/gst/playback/gstplaybin3.c#L30-L157
[gst-playbin-switch]: https://github.com/GStreamer/gstreamer/blob/070125524a8422e29d3b69a372ed4f62fd343ffa/subprojects/gst-plugins-base/gst/playback/gstplaybin3.c#L828-L853
[gst-selector]: https://github.com/GStreamer/gstreamer/blob/070125524a8422e29d3b69a372ed4f62fd343ffa/subprojects/gstreamer/plugins/elements/gstinputselector.c#L1365-L1439
[gst-messages]: https://github.com/GStreamer/gstreamer/blob/070125524a8422e29d3b69a372ed4f62fd343ffa/subprojects/gstreamer/gst/gstmessage.h#L32-L110
[gst-testclock]: https://github.com/GStreamer/gstreamer/blob/070125524a8422e29d3b69a372ed4f62fd343ffa/subprojects/gstreamer/libs/gst/check/gsttestclock.c#L25-L86
[gst-rust-manifest]: https://github.com/GStreamer/gstreamer-rs/blob/4e20c0caa0100421080f081065e51f9341ba4693/Cargo.toml#L121-L127
[gst-rust-readme]: https://github.com/GStreamer/gstreamer-rs/blob/4e20c0caa0100421080f081065e51f9341ba4693/README.md
[gst-manipulation]: https://gstreamer.freedesktop.org/documentation/application-development/advanced/pipeline-manipulation.html
[gst-preroll]: https://gstreamer.freedesktop.org/documentation/additional/design/preroll.html
[gst-live]: https://gstreamer.freedesktop.org/documentation/additional/design/live-source.html
[gst-appsink]: https://gstreamer.freedesktop.org/documentation/app/appsink.html
[gst-appsrc]: https://gstreamer.freedesktop.org/documentation/app/appsrc.html
