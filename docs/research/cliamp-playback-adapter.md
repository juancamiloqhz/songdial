# CLIamp as Songdial's first real playback adapter

Research date: 2026-09-09. Status: original CLIamp assessment; no CLIamp implementation, installation, player execution, or listening validation performed. Subsequent adapter selection is recorded in [ADR 0003](../adr/0003-use-gstreamer-for-real-playback.md).

User disposition, 2026-09-09: preserve Songdial's existing replacement and supersession guarantees and research another adapter. The first real catalog will contain curated public Stations and a small real Track/Playlist catalog without accounts. See the accepted direction in the [product brief](../product.md#direction-after-the-first-milestone) and the [alternative-adapter research](real-playback-adapter-alternatives.md). The findings below remain the original CLIamp assessment.

## Assessment

CLIamp is a credible playback-engine candidate, but its released headless implementation does **not** satisfy Songdial's current replacement and cancellation promises without additional work or an explicit product decision. The transport is considerably more capable than older search results suggest: CLIamp now has versioned IPC, asynchronous operation jobs, runtime snapshots, subscriptions, and editable playback lists. The decisive risks are behavioral, particularly preserving existing playback after a failed replacement and preventing a canceled request from subsequently becoming audible. [IPC documentation][ipc-doc], [daemon implementation][daemon], [daemon job dispatcher][daemon-v2]

Recommendation: take these findings into the planned design interview before writing implementation tickets. If preserving Songdial's existing replacement guarantee is required, decide whether an upstream CLIamp change or researching another playback engine is worth pursuing. A narrow runtime probe can then measure behavior if useful. Merely wrapping successful job responses or ignoring obsolete responses in Songdial cannot make an already-committed audio change disappear.

## Reproducible evidence and limits

- Canonical upstream: [`bjarneo/cliamp`](https://github.com/bjarneo/cliamp), identified by its repository README, module path, and linked first-party site. Its repository declares the MIT license. [README][readme], [module][module], [license][license]
- GitHub's latest-release endpoint returned **v2.2.0**, published **2026-09-08T13:27:58Z**. The annotated tag object is `a086eb82e7c80dc16ebf332a6b5c065850a21d5a`; its peeled source commit is **`af4d03fb0c0f43ee0bcd66551773d83217fce01e`**. All CLIamp source links below pin that commit. [Release](https://github.com/bjarneo/cliamp/releases/tag/v2.2.0), [release API](https://api.github.com/repos/bjarneo/cliamp/releases/latest), [tag](https://github.com/bjarneo/cliamp/tree/v2.2.0)
- Observed upstream `main` was `93a4ac69222a9e172d20b960d4329b73e8c79ecd`, nine commits ahead. The comparison includes changes in `main.go`, playlist context, Jellyfin, resume behavior, and UI files; it includes no changes to `daemon.go`, `daemon_v2.go`, or `ipc/`. This report assesses the release, not an unspecified future `main`. [Pinned comparison](https://github.com/bjarneo/cliamp/compare/af4d03fb0c0f43ee0bcd66551773d83217fce01e...93a4ac69222a9e172d20b960d4329b73e8c79ecd)
- Songdial baseline: **`dc8e62fd266c8e13d97eb040c938a8e528843552`** on `main`. Domain definitions, interaction promises, and architecture references below use that snapshot. [Songdial context][song-context], [interaction model][song-interactions]
- Parent-session verification on 2026-09-09 found that Songdial `main` matched the live remote at that commit. Draft [PR #22](https://github.com/juancamiloqhz/songdial/pull/22) remained open at `6f965e7` and untouched; [issue #11](https://github.com/juancamiloqhz/songdial/issues/11) remained open and `ready-for-human`. The baseline Songdial suite passed **122 tests, including six PTY tests**. This is repository verification, not CLIamp or audible-playback validation.
- Evidence consists of current first-party metadata, a read-only source checkout of v2.2.0, official documentation, and inspection of upstream tests. **CLIamp tests were read, not executed.** No audio device, live Station, real Service account, release binary, or audible output has been verified. Runtime predictions are identified as such.

The temporary source checkout used for research is `/tmp/songdial-cliamp-research-v2.2.0`; the durable evidence is in the pinned links, not that temporary directory. No earlier research directory existed, so this note establishes `docs/research/` as a sensible location without imposing a broader documentation convention.

## Supported control surface

**Documented:** V2 is the only accepted IPC version. Requests and responses are newline-delimited JSON over a local Unix stream socket. Top-level methods are `capabilities`, `state.get`, `spectrum.get`, `operation.submit`, `job.get`, `job.cancel`, and `subscribe`. An operation returns a job; receiving an accepted response is not playback completion. [IPC documentation][ipc-doc], [V2 migration guide][migration]

**Source facts:** Request `id` is echoed; V2 errors carry stable `code` and `message`, with optional diagnostic `detail`. Snapshot fields distinguish actual `track`, `logical_track`, and `playback_detached`. The standard client checks protocol version and response identity. Its default exchange deadline is five seconds, with a three-second connection timeout. [V2 types][v2], [V2 client][v2-client]

The adapter-relevant operations advertised by the release include:

| Purpose | IPC operation and main parameters | Meaning that matters to Songdial |
| --- | --- | --- |
| Playback control | `play`, `pause`, `toggle`, `stop` | Prefer explicit play/pause when reconciling actual state. |
| Play supplied media | `track.play`, `track` | Accepts structured metadata and a local path or supported URI. |
| Queue supplied media | `track.queue`, `track` | Uses CLIamp's play-next behavior; it is not a passive append in every state. |
| Append path | `queue`, `path` | Appends to the live playlist. |
| Inspect/edit live playlist | `queue.list`, `.play`, `.enqueue`, `.remove`, `.move`, `.clear` | Indexes refer to the live playlist, not Songdial's Queue. |
| Inspect/edit play-next list | `playnext.list`, `.remove`, `.move`, `.clear` | Uses its own indexes. |
| Resolve a URL | `url.load`, `path`, `play` | Can expand a feed or remote playlist rather than return one Track. |
| Control automatic order | `shuffle`, `repeat`, `name` | Existing daemon settings affect later playback. |

The operation registry is the source of these names and parameter hints. `playlist.replace` operates on a **saved** playlist; it is not an atomic live-session replacement operation. No prepare/commit replacement transaction appears in the advertised operation set. [Operation registry][operations]

Illustrative raw messages, not commands executed during research:

```json
{"version":2,"id":"capabilities-1","method":"capabilities"}
{"version":2,"id":"state-1","method":"state.get"}
{"version":2,"id":"play-1","method":"operation.submit","operation":"track.play","params":{"track":{"path":"/absolute/path/example.wav","title":"Example"}}}
{"version":2,"id":"events-1","method":"subscribe","topics":["runtime.state","runtime.job"]}
```

Recommendation: a small Rust socket client can consume this public protocol directly; `cliamp remote ...` is useful for manual inspection and a throwaway probe. Discover capabilities at connection time and reject unsupported protocol versions clearly. Do not implement against old `{"cmd":"status"}` examples.

## Track and Station inputs

**Source facts:** CLIamp uses one upstream `Track` type for files and streams. `Stream` indicates HTTP/HTTPS transport; `Realtime` marks a live stream, and `IsLive()` returns that flag. Duration zero means unknown. IPC `TrackInfo` preserves these flags, a required path, metadata, and provider metadata. Therefore an HTTP URL is not sufficient evidence that a Songdial item is a Station. [Playlist types][playlist-types], [IPC track shape][protocol]

Native decoding selects WAV, FLAC, Ogg Vorbis, or MP3; AAC/M4A/ALAC/WMA/Opus/WebM paths need FFmpeg. HLS `.m3u8` is handled by opening the URL in FFmpeg so it can resolve segments. HTTP responses can supply ICY metadata; CLIamp also classifies `icy-*` response headers as live transport metadata. [Decoder][decode], [pipeline][pipeline]

`url.load` uses the argument/URL resolver, which supports files, directories, globs, M3U/PLS, feeds, and recognized external media pages. Consequently arbitrary input URLs may imply network resolution, playlist expansion, or helper processes. A supplied direct Track is the narrower input for a controlled first adapter. [URL resolver][resolver]

**Source and test facts:** Live pause/resume reconnects at the live edge. A drained live Station restarts the same Station rather than advancing to another entry. Upstream tests cover successful restart and a failed restart ending in stopped state. These are not proof of uninterrupted live playback or indefinite reconnect with backoff. [Daemon playback tests][daemon-playback-tests]

Recommendations for the interview:

- Choose explicit finite Track inputs and continuous Station URLs for the first milestone; do not silently claim support for every upstream Service merely because CLIamp lists it.
- Keep Songdial's Track/Station distinction authoritative. Supply `realtime: true` for known Stations; retain Station identity when ICY metadata changes the current artist/title.
- Decide whether live resume means reconnecting now, which is CLIamp's implemented behavior, and how a failed reconnect appears to the Listener.
- Decide what remains Demo catalog data. Fictional Track titles, Sources, or durations must not be presented as the provenance or measured duration of unrelated real audio.

## State, progress, completion, and errors

**Documented/type facts:** Snapshots expose state, actual and logical media, position and duration in seconds, seekability, playlist revisions, playback modes, and `stream_error`. Playing, paused, and stopped are runtime states; there is no typed loading state or distinct EOF reason in `RuntimeSnapshot`. Songdial would own pending-request presentation and its retained Now Playing identity. [V2 snapshot type][v2]

The four retained runtime topics carry snapshots; the daemon publishes only when a state fingerprint changes. Position alone does not change that fingerprint. Subscribe for changes and obtain fresh snapshots for progress; a silent event stream is not evidence of paused playback. The daemon's ordinary tick interval is 250 ms, not a guaranteed event-latency bound under blocking operations. [Daemon runtime publication][daemon-publish], [daemon loop][daemon-loop]

The broker replays the latest retained event for a topic and uses process-local sequence numbers. Slow subscribers receive `system.overflow` and are disconnected. Event payloads are limited to 64 KiB; IPC frames are limited to 1 MiB. A reconnect needs snapshot reconciliation, not replay of an assumed durable log. [Broker][broker], [framing][framing]

Jobs have queued/running/succeeded/failed/canceled states. The store defaults to 256 entries with a 15-minute completed-job TTL. Its internal terminal-event channel may drop newer events when full, so `runtime.job` alone is insufficient for authoritative job accounting; fall back to `job.get`. Request IDs correlate responses; the inspected request shape supplies no idempotency key for safely replaying an operation after an ambiguous disconnect. [Job store][jobs], [request type][v2]

**Source inference:** The daemon observes drained playback and immediately advances or stops before publishing the tick snapshot. Completion is inferred from reconciled state/identity, not received as a dedicated media-ended event. `Player.StreamErr()` describes the current decoder; stopping removes that pipeline. A decode failure and clean EOF can therefore converge on a stopped snapshot without a durable reason. Do not promise that every failure is recoverable from `stream_error`. [Daemon EOF path][daemon-eof], [player error access][player-error], [player stop][player-stop]

Recommendation: model transport failure, command failure, media failure, and backend exit separately from Songdial's play state. Confirm job outcome **and** observed actual playback before displaying success. When outcome is uncertain, resynchronize rather than blindly repeating `track.play`, which can append another live-playlist entry.

## Critical mismatch: replacing the Playback session

Songdial promises that existing playback continues while a replacement loads, success atomically replaces the current item and Queue, failure preserves the old session, and a newer request supersedes an older pending one. [Songdial playback contract][song-interactions]

The released headless path has the following evidence-backed sequence:

1. `track.play` appends the supplied Track and selects it before invoking playback. It returns the normal queue response after that call. [Daemon queue handler][daemon-queue]
2. The player builds the candidate pipeline before swapping it into the speaker, so old audio **can continue during preparation**. On a preparation error, the player returns an error. [Player preparation][player-prepare]
3. The daemon catches that error, logs it, explicitly stops playback, clears the active playback identity, and returns no error to its caller. [Daemon play failure][daemon-failure]
4. The V2 job dispatcher can consequently record a successful job with a stopped snapshot. This is a source-derived failure path, not a runtime observation from this research. [Daemon job completion][daemon-job]

This contradicts failure preservation in the interaction model. A client that displays the old session after the daemon has stopped it would make the interface misleading.

Cancellation has a separate mismatch. Daemon work runs through one control queue. The dispatcher checks cancellation before and after executing the operation, but ordinary `Play`/`PlayYTDL` calls do not accept that job context. A running cancellation can mark the job canceled while the media operation later commits. The interactive TUI uses generation-aware player calls; that protection must not be assumed for `--daemon`. [Daemon execution][daemon-execute], [player generation API][player-prepare], [TUI playback][tui-playback]

Source inference: slow preparation can delay queued pause, stop, state reads, tick processing, and graceful signal handling. An asynchronous job acknowledgement does not imply independently cancellable background playback preparation. Measure this using a controlled delayed endpoint if this route survives the interview.

Possible directions, all **unresolved**:

- Preserve the existing promise and require an upstream headless fix for error propagation, commit ordering, cancellation, and session/Queue replacement.
- Explicitly revise the first real-playback promise to describe interruption and bounded recovery, with honest Listener feedback.
- Research a different backend against the same observable contract.

Two daemons, mute-and-switch, replaying the old Track after failure, or a hidden TUI are not established fixes. Each adds lifecycle, sound-output, or UX behavior that would itself need a design decision and validation.

## Queue ownership is a product decision

CLIamp has a live playlist and a separate priority play-next list. Its `Next()` consumes play-next entries first and then resumes playlist order, subject to shuffle/repeat. Upstream tests explicitly assert that the two lists are separate. Neither is identical to Songdial's Queue of only the ordered Tracks scheduled after the current item. [Playlist advancement][playlist-next], [daemon list tests][daemon-list-tests], [Songdial Queue definition][song-context]

`track.queue` while stopped starts the supplied Track; `queue.clear` stops current audio; `track.play` does not clear prior play-next entries. Multiple ordinary operations do not form a transaction. An `if_revision` guard is compared with the **playlist revision**, and zero acts as no guard. These details rule out a mechanical rename of upstream fields to Songdial concepts. [Queue handler][daemon-queue], [revision check][daemon-execute]

Recommended alternatives to discuss:

| Ownership choice | What it buys | Work still required |
| --- | --- | --- |
| Songdial owns Queue and submits one current Track | Preserves existing domain behavior and avoids importing playlist browsing semantics. | Reliable completion/failure reconciliation; preventing unexpected backend advance; current-item retention; orderly replacement. |
| CLIamp owns active order and Songdial mirrors it | Backend can advance when Songdial is temporarily busy. | Exact mapping of live playlist versus play-next, duplicate identities, mutations, external controls, replacement, and stopped behavior. |

Do not let both Songdial's one-second simulated tick and CLIamp independently advance playback. Pick one owner before implementing the adapter.

## Startup, isolation, and shutdown

**Documented:** `--daemon`/`-d` starts without a TUI, and SIGINT/SIGTERM performs graceful shutdown. Headless mode enables available platform media controls, does not load Lua plugins, and omits gapless preloading. [Headless documentation][headless]

**Source facts:** `CLIAMP_CONFIG_DIR` overrides the configuration root; otherwise lookup uses `XDG_CONFIG_HOME`, `HOME`, and platform fallbacks. The socket path is `<resolved-config-root>/cliamp.sock`. Thus the documentation's one-instance-per-user description really concerns a shared socket path; a private configuration directory permits distinct sockets. This is not proof of complete multi-instance isolation: the separate `DataDir()` helper does not use that override, and OS media integrations remain additional shared resources. [Directory resolution][appdir], [socket path][socket-path]

The IPC server creates the parent directory, applies owner-only socket permissions, writes a PID file, refuses a responsive existing socket, and removes its socket/PID on orderly close. These are useful collision checks; Songdial still needs ownership of the specific child it launches. [IPC server lifecycle][server], [stale socket probe][socket-stale]

Bare startup defaults to a radio provider and can synchronously fetch its built-in playlist in daemon mode before creating the audio engine and IPC server. Audio-device initialization can fail before IPC becomes available. An isolated config with `--provider local --no-auto-play --no-shuffle --repeat off` is a **proposed** controlled launch configuration, not a verified command sequence. Readiness should mean a successful version/capability/state exchange, not a fixed sleep or the startup log line. [Startup][startup], [CLI flags][flags]

Recommendations: decide whether Songdial owns a private child or attaches to a Listener-managed instance, whether audio continues after Songdial exits, and whether media-key changes should be reflected in Songdial. For an owned child, reap it on shutdown and use a bounded escalation policy directed only at that child. Never kill an unrelated daemon by executable name.

## Installation and platform assumptions

The v2.2.0 release supplies Linux amd64/arm64, macOS amd64/arm64, and Windows amd64 artifacts plus checksums; Windows has both a standalone executable and a ZIP. This covers Songdial's current Linux x86_64 and two macOS packaging targets at the artifact level, not a verified combined installation. [Release](https://github.com/bjarneo/cliamp/releases/tag/v2.2.0), [CLIamp release build][release-build], [Songdial packaging workflow][song-ci]

Official installation notes say macOS prebuilt binaries need Homebrew FLAC, Vorbis, Ogg, and mpg123 libraries; the Homebrew formula installs them. Linux release codecs are statically linked, but an ALSA bridge may be needed for PipeWire/PulseAudio. Optional FFmpeg and yt-dlp requirements depend on chosen formats and external inputs. Windows ZIP packages include codec DLLs. [Installation documentation][readme]

There is documentation drift: the release README says Go 1.25.5 or later, while the release's `go.mod` declares **Go 1.26.6**. A reproducible source build must follow the actual module/toolchain requirements. Upstream CI reads the Go version from `go.mod` and installs native codec dependencies. [Module][module], [CI][ci]

`install.sh` selects a release through `/releases/latest`, requires a matching SHA-256 checksum, installs into a selected bin directory, and adds Linux desktop/icon/URL-handler entries. It is not a pinned, side-effect-free dependency fetch. Recommendation: choose and document an explicit CLIamp version if packaged with Songdial; otherwise detect compatible installed versions and provide concrete setup guidance. No installer was run. [Installer][installer]

## Deterministic testing and a bounded prototype

Upstream tests demonstrate useful seams: fake `player.Engine` implementations for daemon state/jobs and list behavior, in-process IPC servers, controlled HTTP servers, injected stream failures, and fake FFmpeg executables. Its CI runs the Go suite with race detection on Linux and tests/builds on macOS and Windows. These are source-level test intentions; this research did not run them or verify any particular CI run. [Daemon tests][daemon-list-tests], [IPC lifecycle tests][ipc-tests], [HTTP decoder tests][decode-tests], [FFmpeg tests][ffmpeg-tests], [CI][ci]

Proposed Songdial validation layers:

1. **Deterministic core:** fake playback observations and commands; assert replacement request ordering, pending/failure UI, pause confirmation, Queue policy, Station recovery, and backend events while the terminal is below minimum size.
2. **Protocol contract:** fake local socket server; assert NDJSON framing, optional fields, correlated responses, accepted-versus-completed jobs, late results, disconnects, duplicate events, retained snapshots, overflow, and reconciliation after backend restart.
3. **Controlled real process:** pinned binary, private configuration, generated short local WAVs, and a local HTTP server that can delay, truncate, stall, or reject a request. Record raw job/state/event transcripts and process exit timing.
4. **Audible/platform check:** verify actual output on supported Listener machines using the packaged build and chosen real catalog inputs. Passing protocol tests or observing an increasing position is not proof that a Listener heard sound.

If the interview keeps CLIamp under consideration, the first prototype should answer one narrow question: **Can the released headless process uphold Songdial's replacement promise under failed and canceled slow loads?**

Use A as a currently playing deterministic Track; request B from a controlled endpoint that delays then fails; repeat with delayed-success B canceled after its job begins, then request C. Capture whether A keeps playing, B ever becomes active after cancellation, which job states and diagnostics are visible, and how quickly pause/state/shutdown respond. Reproduce only this contract; do not build a second Songdial UI or silently promote prototype code. A null output sink can settle protocol and ordering questions, but audible continuity requires a real-output observation.

The source already establishes a contract mismatch; a prototype can confirm its manifestation and scope. It should not be presented as needed to discover whether the daemon calls `Stop()` on preparation failure—that is visible in the pinned code.

## Songdial seam and next interview

Songdial already has a narrow `LoadPlayback` effect and request-tagged load results, but its application owns simulated progress, pause, and Queue advance. The terminal runtime completes loads on a timer; catalog Tracks and Stations have no real locator. A genuine adapter therefore needs commands and observations beyond replacing one load callback. [Application][song-app], [terminal runtime][song-terminal], [catalog][song-catalog]

Recommendation: keep navigation, selection, Listening intent, Source identity, and Songdial's Playback session in the application core; keep process/socket I/O and backend reconciliation at its effect boundary. Map catalog identity to a concrete locator through the first demonstrated integration without adopting CLIamp's provider metadata as the permanent Songdial domain model. This is consistent with the ADR deferring a generalized Service framework until a real integration shows what varies. [Foundation ADR][song-adr]

Resolve these questions in order:

1. Is preserving the old session and suppressing superseded requests still required for the first audible milestone?
2. Which real finite Tracks and continuous Stations are promised, and which browsing content stays visibly Demo catalog data?
3. Who owns Queue advancement, and what happens when a Track fails or a Station disconnects?
4. Does Songdial launch/stop a private CLIamp child, or attach to an existing instance? What happens when CLIamp is absent, incompatible, busy, or exits?
5. Which platforms and installation path must be usable before the deferred Listener study resumes?

Only after those choices and any narrowly required runtime evidence should the design thread become a milestone specification and dependency-linked implementation tickets. This note supplies no participant observations or usability verdict and does not change the deferred validation state or draft PR #22.

[ipc-doc]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/docs/remote-control.md
[migration]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/docs/upgrading-ipc-v2.md
[headless]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/docs/headless.md
[readme]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/README.md
[module]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/go.mod#L1-L3
[license]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/LICENSE
[v2]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/v2.go#L39-L131
[v2-client]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/v2_client.go#L12-L54
[operations]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/operations.go#L118-L190
[protocol]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/protocol.go#L87-L109
[playlist-types]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/playlist/playlist.go#L36-L60
[playlist-next]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/playlist/playlist.go#L711-L770
[decode]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/player/decode.go#L190-L335
[pipeline]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/player/pipeline.go#L233-L318
[resolver]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/resolve/resolve.go#L75-L120
[daemon]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon.go
[daemon-loop]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon.go#L89-L109
[daemon-eof]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon.go#L308-L329
[daemon-failure]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon.go#L360-L387
[daemon-queue]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon.go#L572-L627
[daemon-v2]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon_v2.go
[daemon-job]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon_v2.go#L147-L189
[daemon-execute]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon_v2.go#L192-L273
[daemon-publish]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon_v2.go#L602-L662
[player-prepare]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/player/player.go#L183-L283
[player-stop]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/player/player.go#L479-L519
[player-error]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/player/player.go#L1015-L1027
[tui-playback]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ui/model/playback.go#L363
[jobs]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/jobs.go
[broker]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/pubsub.go#L12-L135
[framing]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/framing.go#L8-L14
[appdir]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/internal/appdir/appdir.go#L9-L60
[socket-path]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/client.go#L10-L17
[server]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/server.go#L122-L198
[socket-stale]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/server.go#L547-L580
[startup]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/main.go#L293-L352
[flags]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/commands.go#L28-L68
[installer]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/install.sh
[release-build]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/.github/workflows/release.yml#L12-L31
[ci]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/.github/workflows/ci.yml
[daemon-playback-tests]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon_playback_test.go#L63-L124
[daemon-list-tests]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/daemon_v2_test.go#L15-L200
[ipc-tests]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/ipc/server_lifecycle_test.go
[decode-tests]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/player/decode_test.go#L45
[ffmpeg-tests]: https://github.com/bjarneo/cliamp/blob/af4d03fb0c0f43ee0bcd66551773d83217fce01e/player/ffmpeg_test.go#L305-L433
[song-context]: https://github.com/juancamiloqhz/songdial/blob/dc8e62fd266c8e13d97eb040c938a8e528843552/CONTEXT.md
[song-interactions]: https://github.com/juancamiloqhz/songdial/blob/dc8e62fd266c8e13d97eb040c938a8e528843552/docs/interaction-model.md#playback-and-queue
[song-app]: https://github.com/juancamiloqhz/songdial/blob/dc8e62fd266c8e13d97eb040c938a8e528843552/src/application.rs
[song-terminal]: https://github.com/juancamiloqhz/songdial/blob/dc8e62fd266c8e13d97eb040c938a8e528843552/src/terminal.rs
[song-catalog]: https://github.com/juancamiloqhz/songdial/blob/dc8e62fd266c8e13d97eb040c938a8e528843552/src/catalog.rs
[song-adr]: https://github.com/juancamiloqhz/songdial/blob/dc8e62fd266c8e13d97eb040c938a8e528843552/docs/adr/0001-build-the-first-milestone-as-a-durable-foundation.md
[song-ci]: https://github.com/juancamiloqhz/songdial/blob/dc8e62fd266c8e13d97eb040c938a8e528843552/.github/workflows/package.yml
