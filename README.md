# Songdial

> Tune into the right music.

Songdial is a discovery-first terminal music application. It brings moods,
radio stations, personal playlists, and search into one coherent interface,
independent of where the music comes from.

Songdial is in pre-alpha. The Rust and Ratatui application opens directly on
Home and includes complete browse journeys for Mood & activity, Radio stations,
My playlists, and both fictional Service catalogs, with read-only details from
a deterministic Demo catalog. Tracks, Playlists, and Stations can start silent
simulated Playback sessions with persistent Now Playing state. Tracks and
Playlists can be appended to a scrollable Queue where upcoming Tracks can be
inspected, removed, or played. Search everything provides incremental,
Source-aware results across the complete Demo catalog.

## Build and run

Install the repository-pinned Rust toolchain, then build Songdial from a clean
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

- `↑`/`k` and `↓`/`j` move selection and scroll long lists.
- `Enter` opens the selected choice, Station, or Playlist without starting
  playback.
- `p` starts the selected Track, Playlist, or Station as a new simulated
  Playback session.
- `a` appends a selected Track or a Playlist's playable Tracks to the Queue.
- `n` opens expanded Now Playing and its scrollable Queue.
- `d` removes the selected queued Track.
- `Space` pauses or resumes playback and restarts a retained stopped Track.
- `/` opens Search or restores focus to its query.
- Printable input edits a focused Search query; `Backspace` erases and `Esc`
  returns to result navigation.
- `Esc` restores the previous Destination's exact selection and scroll position.
- `?` opens contextual help.
- `q` quits, with confirmation while a Playback session or Queue is active;
  `Ctrl+C` always exits immediately.

Songdial restores raw mode, cursor visibility, and the alternate screen when it
exits.

## Test

Run the complete automated suite with:

```sh
cargo test --locked
```

Standalone release binaries, target-specific packaging commands, artifact
verification, and the `80×24` terminal support contract are documented in
[the release guide](docs/release.md).

## Product documents

- [Product brief](docs/product.md)
- [Interaction model](docs/interaction-model.md)

## Status

Pre-alpha. Home, the durable Application Module/terminal adapter foundation,
the fixed Demo catalog, Mood & activity, Radio stations, My playlists,
both fictional Service catalogs, deterministic simulated Playback sessions, and
interactive Queue management, and Source-aware grouped Search are implemented.
Networking and persistence are not yet implemented.
