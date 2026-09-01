# Songdial interaction model

## Purpose

This document defines the observable behavior of Songdial's first executable
milestone. It describes what the Listener can see and do without prescribing
the final internal package structure.

## Application frame

Every Destination shares four regions:

1. **Location** — the current Destination and breadcrumb trail.
2. **Browser** — the catalog, details, results, or Queue currently being
   explored.
3. **Now Playing** — persistent playback summary with title, Source, progress,
   and play state.
4. **Key guide** — concise, contextual help for available actions.

The Browser owns most of the terminal area. The other regions preserve context
without competing with the current task.

At `80×24`, Location occupies one row, Browser the next 19 rows, Now Playing
two rows, and the key guide the final two rows. At wide sizes, the Browser adds
a read-only detail lens beside its list; the lens never becomes a focus target
or changes the available keys. Color reinforces state but never carries it
alone.

## Navigation model

Songdial uses a navigation stack of Destination snapshots:

- `Enter` opens the selected Destination without playing it.
- `Esc` restores the previous Destination's exact selection, scroll offset,
  active pane, and Destination-local query state.
- `Esc` on Home has no destructive effect.
- `q` requests quit from every Destination. Confirmation appears only while a
  Playback session is active or the Queue is nonempty.
- `Ctrl+C` exits immediately while still restoring the terminal.
- No Service, Playable item, Search result, or playback state may replace or
  disable this navigation behavior.

Selection and playback are independent states. Moving the selection never
changes what is playing. Opening an item never starts it automatically. Dense
lists reserve textual state and Source columns before truncating titles so the
Listener can always distinguish `SELECTED`, `PLAYING`, their combination, and
`UNAVAIL` without relying on color.

## Global keys

| Key       | Behavior                                          |
| --------- | ------------------------------------------------- |
| `↑` / `k` | Move selection up                                 |
| `↓` / `j` | Move selection down                               |
| `Enter`   | Open selected Destination or Playable item        |
| `p`       | Start a new Playback session                      |
| `Space`   | Toggle play and pause                             |
| `a`       | Add a Track or Playlist to the Queue              |
| `d`       | Remove the selected queued Track                  |
| `/`       | Open Search or focus its query                    |
| `n`       | Open Now Playing and Queue                        |
| `Esc`     | Restore the previous Destination or close overlay |
| `?`       | Show contextual help                              |
| `q`       | Request quit                                      |

Keys may be shown or hidden in the guide based on whether their action is valid,
but their meaning must remain stable throughout the application. The persistent
guide shows about five context-valid actions; `?` provides the complete global
and local key set. While the Search query has focus, printable letters are query
input rather than global commands.

## Destinations

### Home

Home presents:

- Mood & activity
- Radio stations
- My playlists
- Browse services
- Search everything

Selection begins on Mood & activity. Continue Listening and persisted history
are outside this milestone.

### Mood & activity (Listening intents)

This Destination shows Listening intents such as Deep Work, Focus, Flow, Calm,
Energy, and Reset. Opening one reveals its available Stations and Playlists
along with a short explanation of the intended listening character.

A Listening intent is Songdial curation, not a Service or Playable item.

### Radio stations

This Destination shows continuous Stations. Each entry communicates its name,
style, Source, and availability. Opening a Station shows details; `p` starts a
simulated continuous Playback session.

### My playlists

This Destination groups personal and saved Playlists. Source information is
visible but secondary. Opening a Playlist reveals its description and Tracks.

### Browse services

This Destination exposes Service-specific catalogs deliberately, using
Source-filtered sections for Stations, Playlists, and Tracks. Listening intents
remain Songdial-owned curation. Returning with `Esc` always restores the
previous Songdial Destination rather than a Service-defined parent.

### Search

Search is its own Destination. Results update incrementally while the Listener
types and appear in nonempty groups ordered by Listening intents, Stations,
Playlists, and Tracks. Case-insensitive token matching ranks exact titles,
title prefixes, then other title, descriptive-tag, creator, and Source matches.
Source badges remain visible, including on apparent cross-Service duplicates.
Services act as filters rather than Search results. Opening a result never
starts playback, and query entry never erases Now Playing or navigation state.

### Now Playing and Queue

This Destination expands the persistent summary. It shows the current or
retained item first with Source, play state, and progress, followed by a
scrollable Queue with its count and position. Queue selection supports `Enter`
for Track details, `p` for a new Playback session, and `d` for removal. An empty
Queue explains how to add Tracks. Browsing elsewhere does not close or reset it.

## Playback and Queue

`p` starts a new Playback session deliberately:

- A Track becomes current with an empty Queue.
- A Playlist starts its first available Track and queues the remainder.
- A Station starts a continuous session with an empty Queue.

The existing session continues while a replacement loads. Success replaces the
current item and Queue atomically; failure preserves the existing session and
shows the reason. A newer playback request supersedes an older pending request.

`a` appends a Track or a Playlist's available Tracks in order. Stations cannot
be queued, so `a` is unavailable for them. `Space` toggles playing and paused;
from stopped state it restarts the retained Track. Tracks advance with a
deterministic one-second clock and proceed through the Queue. An exhausted Queue
leaves the final Track visible as stopped. Stations display `LIVE` and never
finish automatically.

## Feedback and motion

Transitions should be brief, interruptible, and connected to state changes:

- A short tuning treatment may accompany starting a Playback session or
  changing Services.
- Loading indicators communicate actual waiting.
- Now Playing changes may use a restrained reveal to make the new state clear.
- Navigation must remain usable when animation is disabled.

Navigation itself is instantaneous. Animation must never delay keyboard input
or disguise an unavailable action. `--no-motion` disables motion, while
`NO_COLOR` removes color independently. Success feedback such as Queue changes
briefly replaces the key guide; failures remain visible inline until the next
relevant action.

## Responsive behavior

The milestone is fully usable at `80×24`. A wide Browser adds a read-only detail
lens beside its list while preserving the same Destination, selection, keys,
and navigation. The exact wide breakpoint remains a content-width decision;
intermediate widths require verification during implementation.

Below `80×24`, the application frame is replaced rather than squeezed. A clear
message reports current and required dimensions, allows quit, and restores the
unchanged session when the terminal is resized back. Resizing at any supported
size preserves selection, scroll offset, active pane, query, Playback session,
and Queue.

## Demo catalog

The fixed Demo catalog contains enough realistic data to expose design
problems:

- six Listening intents;
- eight Stations across several styles;
- five Playlists, including one intentionally empty Playlist;
- 24 source-scoped Tracks across two fictional Services;
- two apparent cross-Service duplicates;
- overlapping Search results;
- available, loading, empty, and unavailable states;
- a Queue long enough to require scrolling.

The Demo catalog resets on each launch and keeps stable identities, ordering,
durations, and loading scenarios. It is a design instrument, not the beginning
of a permanent Service integration format.

## Required usability properties

- The Listener can always tell what is selected and what is playing, including
  without color.
- `Esc` behavior is predictable.
- Opening and playing are never conflated.
- Source information is available without dominating navigation.
- Empty and unavailable states explain what happened and what can be done next.
- Essential actions are discoverable from the application frame.
- Wide layout adds detail without adding focus or navigation state.
- Resizing never resets the Listener's session.

## Prototype validation

The persistent frame passed its mechanical prototype gate at `80×24`,
`120×40`, under `NO_COLOR`, and through below-minimum resize recovery. The
immutable [prototype findings](https://github.com/juancamiloqhz/songdial/blob/8e926d97a21961d924de3487e55fc77ee5ec9fb7/prototype/persistent-frame/FINDINGS.md)
and [screenshot evidence](https://github.com/juancamiloqhz/songdial/tree/8e926d97a21961d924de3487e55fc77ee5ec9fb7/prototype/persistent-frame/screenshots)
remain primary sources on the throwaway prototype branch; the prototype code is
not production code and must not be merged or promoted.

The prototype did not settle the wide breakpoint, breadcrumb ellipsis,
localized or complex-script text behavior, terminal-theme contrast, real
loading and playback errors, resize storms, or human usability. Those remain
implementation or usability-validation work.
