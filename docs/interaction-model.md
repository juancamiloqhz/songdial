# Songdial interaction model

## Purpose

This document defines the observable behavior of Songdial's initial interactive
prototype. It describes what the listener can see and do without prescribing
the final internal package structure.

## Application frame

Every primary screen shares four regions:

1. **Location** — the current destination and breadcrumb trail.
2. **Browser** — the collection, details, results, or queue currently being
   explored.
3. **Now Playing** — persistent playback summary with title, source, progress,
   and play state.
4. **Key guide** — concise, contextual help for available actions.

The browser owns most of the terminal area. The other regions preserve context
without competing with the current task.

## Navigation model

Songdial uses a navigation stack:

- `Enter` opens the selected destination.
- `Esc` returns to the previous destination.
- `Esc` on Home has no destructive effect.
- `q` quits from Home and asks for confirmation elsewhere only if an operation
  would be lost.
- No provider, collection, search result, or playback state may replace or
  disable this navigation behavior.

Selection and playback are distinct states. Moving the selection never changes
what is playing. Opening an item never starts it automatically.

## Global keys

| Key | Behavior |
| --- | --- |
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Open selected destination |
| `p` | Play selected playable item |
| `Space` | Toggle play and pause |
| `a` | Add selected playable item to the queue |
| `/` | Open global search |
| `n` | Open Now Playing and queue |
| `Esc` | Go back or close the active overlay |
| `?` | Show contextual help |
| `q` | Quit from Home |

Keys may be shown or hidden in the guide based on whether their action is valid,
but their meaning must remain stable throughout the application.

## Destinations

### Home

Home presents:

- Mood & activity
- Radio stations
- My playlists
- Browse services
- Search everything

It may also show a small Continue Listening section once history exists. The
first prototype does not require persisted history.

### Mood & activity

This destination shows curated intentions such as Deep Work, Focus, Flow, Calm,
Energy, and Reset. Opening one reveals its available stations, playlists, or
mixes along with a short explanation of the intended listening character.

Mood is metadata and curation, not a playback provider.

### Radio stations

This destination shows continuous streams. Each entry communicates its name,
style, source, and availability. Opening a station shows details; `p` starts
simulated playback.

### My playlists

This destination groups personal and saved playlists. Source information is
visible but secondary. Opening a playlist reveals its description and tracks
when those details are available.

### Browse services

This destination exposes provider-specific catalogs deliberately. Returning
with `Esc` always restores the previous Songdial destination rather than a
provider-defined parent.

### Search

Search is global by default. Results are grouped by useful result kind—stations,
playlists, tracks, and collections—with source badges where needed. Query entry
must not erase Now Playing or the navigation stack.

### Now Playing and queue

This destination expands the persistent playback summary. It shows the current
item, its source, play state, and queued items. Browsing elsewhere does not close
or reset it.

## Feedback and motion

Transitions should be brief, interruptible, and connected to state changes:

- A short tuning treatment may accompany changing streams or providers.
- Loading indicators communicate actual waiting.
- Now Playing changes may use a restrained reveal to make the new state clear.
- Navigation must remain usable when animation is disabled.

Animation must never delay keyboard input or disguise an unavailable action.

## Responsive behavior

The prototype should remain usable in a compact terminal. Wide layouts may show
details beside a list; narrow layouts should show one primary region at a time
while preserving the Now Playing summary and navigation behavior.

## Demo catalog

The first prototype should contain enough realistic data to expose design
problems:

- at least six moods or activities;
- at least eight radio stations across several styles;
- at least four playlists from more than one notional source;
- overlapping search results;
- available, loading, empty, and unavailable states;
- a queue long enough to require scrolling.

The demo catalog is a design instrument, not the beginning of a permanent
provider format.

## Required usability properties

- The listener can always tell what is selected and what is playing.
- `Esc` behavior is predictable.
- Opening and playing are never conflated.
- Source information is available without dominating navigation.
- Empty and unavailable states explain what happened and what can be done next.
- Essential actions are discoverable from the application frame.
