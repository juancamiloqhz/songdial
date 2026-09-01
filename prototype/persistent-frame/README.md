# Songdial persistent-frame prototype

> THROWAWAY PROTOTYPE - preserve this branch as evidence; do not promote this
> crate into production or merge it into `main`.

This Ratatui artifact answers one question:

> Does Songdial's fixed persistent frame preserve context without crowding the
> Browser at `80x24`, while a wide terminal adds useful detail without changing
> navigation?

Run it from the repository root:

```sh
cargo run --manifest-path prototype/persistent-frame/Cargo.toml
```

The fixed demo walkthrough uses `j`/`k` (or arrow keys), `Enter`, `Esc`, `p`,
`a`, `Space`, `/`, `n`, `d`, `?`, and `q`. Search for `night` to expose grouped
cross-Source results. Set `NO_COLOR=1` to exercise the textual state treatment.

The executable also has two evidence helpers. Both drive the same in-memory
state transitions and the same `render` function as the interactive TUI:

```sh
cargo run --manifest-path prototype/persistent-frame/Cargo.toml -- --verify-walkthroughs
cargo run --manifest-path prototype/persistent-frame/Cargo.toml -- --capture-matrix
```

## Visual plan

The subject is a calm broadcast instrument for keyboard-oriented listeners; its
single job is to keep browsing context and the active Playback session legible
at once.

- Color: deck charcoal `#101315`, warm paper `#E6DDC9`, dial amber `#D69A3A`,
  signal green `#65B97B`, muted brass `#857B69`, and slate `#202629`.
- Type: the host monospace is unavoidable in a terminal, so roles come from
  treatment: restrained uppercase/bold display labels, sentence-case body
  copy, and dim uppercase utility telemetry.
- Layout: compact is a strict `1 / flexible / 2 / 2` row stack; wide preserves
  that order and adds only a read-only Browser detail lens.
- Signature: a fixed-width tuning rail names `SELECTED`, `PLAYING`, and
  `UNAVAIL` independently before the item title. Source badges own a protected
  right column, so neither state nor provenance can disappear through title
  truncation.

Decorative boxes and animated meters were intentionally removed: they consumed
the exact compact rows this prototype is meant to evaluate without clarifying a
state transition.
