# Songdial

> Tune into the right music.

Songdial is a discovery-first terminal music application. It brings moods,
radio stations, personal playlists, and search into one coherent interface,
independent of where the music comes from.

Songdial is in pre-alpha. The Rust and Ratatui application opens directly on
Home and includes its first complete browse journey: Mood & activity leads to
six choices for what fits right now, matching Stations and Playlists, and
read-only details from a deterministic fictional Demo catalog. Tracks,
Playlists, and Stations can start silent simulated Playback sessions with
persistent Now Playing state. The remaining Home choices stay as explicit
placeholder Destinations while later Search and Queue-management work remains
intentionally out of scope.

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

- `↑`/`k` and `↓`/`j` move selection and scroll long lists.
- `Enter` opens the selected choice, Station, or Playlist without starting
  playback.
- `p` starts the selected Track, Playlist, or Station as a new simulated
  Playback session.
- `Space` pauses or resumes playback and restarts a retained stopped Track.
- `Esc` restores the previous Destination's exact selection and scroll position.
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

Pre-alpha. Home, the durable Application Module/terminal adapter foundation,
the fixed Demo catalog, Mood & activity browsing, and deterministic simulated
Playback sessions are implemented. Search, interactive Queue management,
networking, and persistence are not yet implemented.
