# Songdial

> Tune into the right music.

Songdial is a discovery-first terminal music application. It brings moods,
radio stations, personal playlists, and search into one coherent interface,
independent of where the music comes from.

Songdial is in pre-alpha. Its first production tracer bullet is a Rust and
Ratatui terminal application that opens directly on Home. Every Home choice can
be explored through an explicit placeholder Destination while later catalog and
playback work remains intentionally out of scope.

## Build and run

Install a current stable Rust toolchain, then build Songdial from a clean
checkout with:

```sh
cargo build --locked
```

Run the application in an interactive terminal:

```sh
cargo run --locked
```

Disable motion treatments when desired:

```sh
cargo run --locked -- --no-motion
```

The built executable is `target/debug/songdial`. Use `songdial --help` for the
complete command-line surface.

## Keyboard controls

- `↑`/`k` and `↓`/`j` move the Home selection.
- `Enter` opens the selected Destination without starting playback.
- `Esc` returns Home and restores its selection.
- `?` opens contextual help.
- `q` or `Ctrl+C` quits.

Songdial restores raw mode, cursor visibility, and the alternate screen when it
exits.

## Test

Run the complete automated suite with:

```sh
cargo test --locked
```

## Product documents

- [Product brief](docs/product.md)
- [Interaction model](docs/interaction-model.md)

## Status

Pre-alpha. Home and the durable Application Module/terminal adapter foundation
are implemented; catalog browsing, playback, networking, and persistence are
not yet implemented.
