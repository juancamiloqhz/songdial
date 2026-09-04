# Standalone binaries

Songdial packages one standalone executable for each supported target:

| Target | Artifact |
| --- | --- |
| macOS arm64 | `songdial-aarch64-apple-darwin` |
| macOS x86-64 | `songdial-x86_64-apple-darwin` |
| Linux x86-64 | `songdial-x86_64-unknown-linux-gnu` |

The executable uses the fixed Demo catalog and simulated playback. It does not
need credentials, network access, a Service account, or an audio device.

## Build and test locally

Songdial pins Rust 1.98.0 in `rust-toolchain.toml`. From a clean checkout:

```sh
cargo check --locked --all-targets --all-features
cargo test --locked --all-targets --all-features
```

To repeat just the packaged-executable acceptance suite:

```sh
cargo test --locked --test cli
cargo test --locked --test terminal_smoke
```

## Package and verify an artifact

Run packaging on a host that matches the target operating system and
architecture. Install the target, build the named release binary, and verify it:

```sh
rustup target add x86_64-unknown-linux-gnu
./scripts/package.sh x86_64-unknown-linux-gnu dist
./scripts/verify-artifact.sh \
  dist/songdial-x86_64-unknown-linux-gnu \
  x86_64-unknown-linux-gnu
```

Replace the target triple with `aarch64-apple-darwin` or
`x86_64-apple-darwin` on the corresponding macOS host. The verifier rejects
an unexpected filename or architecture, checks `--help` and `--version`,
and runs the complete PTY journey with `--no-motion`. That journey observes
Home, browse, Search, playback, Queue, the below-minimum guard, recovery, and
quit. It also checks terminal restoration after normal exit, `Ctrl+C`, a
partially initialized startup failure, and a handled runtime failure.

## GitHub Actions

The **Package standalone binaries** workflow runs for a `v*` tag or a manual
`workflow_dispatch`. Its three native jobs use the pinned Rust toolchain and
locked Cargo dependency graph, run the complete test suite, package one named
binary, verify it on its target architecture, and upload it as a workflow
artifact for 30 days.

The workflow only creates workflow-run artifacts. It does not create a tag or
publish a GitHub Release.

## Terminal support

Songdial is fully supported at `80×24` cells and larger. Below that minimum,
the application replaces the frame with a guard showing the current and
required dimensions. Resize back to at least `80×24` to recover the unchanged
Destination, selection, Search query, Playback session, and Queue. While the
guard is visible, `q` still requests quit (and asks for confirmation when a
Playback session or Queue is active); `Ctrl+C` exits immediately. Both paths
restore raw mode, cursor visibility, and the alternate screen.
