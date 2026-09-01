# Persistent frame prototype findings

## Question and verdict

> Does Songdial's fixed persistent frame preserve context without crowding the
> Browser at `80x24`, while a wide terminal adds useful detail without changing
> navigation?

**PASS**

The fixed frame remained usable at `80x24`; the `120x40` treatment added a
read-only detail lens without changing destinations, keys, selection, scroll,
Search query, Queue, or Playback session.

## Gate evidence

### 1. Persistent regions do not overlap at `80x24`

PASS. The compact renderer assigns exact rows: Location `0`, Browser `1-19`,
Now Playing `20-21`, and key guidance `22-23`. The deterministic walkthrough
check asserts the region labels at those boundaries. The separation is visible
in [Home](screenshots/80x24-home.png),
[Intent + playback](screenshots/80x24-intent-playing.png),
[Search](screenshots/80x24-search.png), and
[Queue](screenshots/80x24-queue.png).

Live walkthrough observation: the real Crossterm process in an `80x24` tmux
window kept the Now Playing strip and guide on the same fixed rows while the
Browser changed among Home, Deep Work, Track details, Search, and Queue.

### 2. Essential Browser content survives truncation

PASS. Every selectable row protects an 11-cell textual state rail on the left
and an 11-cell Source badge on the right; only the middle kind/title field may
clip. [Intent + playback](screenshots/80x24-intent-playing.png) simultaneously
shows `PLAYING` Night Ledger from `[MORROW]` and `SELECTED` Daylight Circuit from
`[HARBOR]`. [Search](screenshots/80x24-search.png) keeps both apparent Night
Geometry duplicates Source-scoped. The unavailable Harbor Track is labelled
`UNAVAIL`, and opening it gives its explicit reason.

Compact breadcrumbs, descriptive subtitles, and the trailing prototype notice
may clip. No observed clipping hid selection, playback state, availability, or
necessary Source identity.

### 3. Selection and playback remain distinct, including `NO_COLOR`

PASS. Selection uses `SELECTED >`; playback uses `PLAYING *`; a shared row uses
`SEL+PLAY >`; unavailable entries use `UNAVAIL !`. These labels supplement amber
selection and signal-green playback. The monochrome capture preserves the same
labels and uses reverse/bold treatment without any explicit RGB cells:
[NO_COLOR Intent](screenshots/80x24-intent-no-color.png).

The persistent strip always names `[STOPPED]`, `[PLAYING]`, `[PAUSED]`, or
`[LIVE]`. Moving selection never changed the strip during either automated or
live walkthroughs.

### 4. Required Destinations are keyboard-usable at both target sizes

PASS. Home, Deep Work, Search, and Now Playing/Queue use the same keys and state
transitions at both sizes. The paired captures preserve the same logical states:

- Home: [compact](screenshots/80x24-home.png) / [wide](screenshots/120x40-home.png)
- Intent + playback: [compact](screenshots/80x24-intent-playing.png) / [wide](screenshots/120x40-intent-playing.png)
- Search: [compact](screenshots/80x24-search.png) / [wide](screenshots/120x40-search.png)
- Queue: [compact](screenshots/80x24-queue.png) / [wide](screenshots/120x40-queue.png)

The live walkthrough completed Home -> Listening intent -> Playlist details ->
`p`; global Search with incremental `night` results; result opening; Now Playing
opening; Queue scrolling; `d` removal; and returns with `Esc`.

### 5. `Esc` restores exact prior navigation state

PASS. The compact Deep Work live walkthrough returned to `ITEM 17/20`,
`OFFSET 1`, `PANE LIST` after moving inside the child Playlist Destination. The
Search walkthrough opened the Harbor Night Geometry Track without playback and
returned to query `night`, `ITEM 7/8`, `OFFSET 0`, `PANE LIST`. Returning from
Queue restored that same Search state. The deterministic verifier compares the
complete stored navigation snapshot, including query text.

### 6. Resize preserves state and wide mode changes density only

PASS. One unchanged live Search state was resized `80x24 -> 120x40 -> 79x23 ->
80x24`. Before and after, it retained query `night`, selected Harbor result
`7/8`, list pane, Morrow Playback session, and the post-removal Queue count of
18. At `120x40`, only the read-only detail lens appeared; navigation and keys did
not change. The deterministic verifier also compares an application-state
fingerprint across all four renders.

### 7. Minimum-size guard recovers unchanged state

PASS. At `79x23`, the frame is wholly replaced by a message that reports both
dimensions and exposes resize recovery plus `q`: [minimum-size
guard](screenshots/79x23-too-small.png). Resizing the live process back to
`80x24` restored the exact prior Search and Playback state. Navigation input is
ignored while guarded, so only resize or quit can affect the session.

## Frame measurements and density choices

| Measurement | Compact `80x24` | Wide `120x40` |
| --- | ---: | ---: |
| Location | 1 row | 1 row |
| Browser | 19 rows x 80 columns | 35 rows x 120 columns |
| Now Playing | 2 rows | 2 rows |
| Key guide + state telemetry | 2 rows | 2 rows |
| Browser list | 80 columns | 57 columns |
| Wide separator | absent | 1 column |
| Wide detail lens | absent | 62 columns |
| Protected row state rail | 11 columns | 11 columns |
| Protected Source badge | 11 columns | 11 columns |
| Now Playing state / Source | 23 / 11 columns | 23 / 11 columns |

Deep Work uses three Browser header rows, leaving 16 compact item rows. Selecting
item 17 therefore produces offset 1 while keeping the playing item visible. Now
Playing/Queue uses five header/current rows, leaving 14 compact Queue rows; the
20-Track Playlist creates a 19-Track Queue, and the captured post-removal state
has 18 Tracks at offset 4. Search fits four group headings and eight concrete
results without scrolling.

The wide split intentionally gives the selected item a full detail treatment
rather than simply stretching every row. Some unselected list titles clip
earlier in the 57-column list, but the state and Source columns remain protected
and the selected item's full identity appears in the 62-column lens.

## Decisions ready for `/to-spec`

1. Keep the persistent `1 / flexible / 2 / 2` vertical order as the compact and
   wide frame baseline.
2. Treat selection and playback as independent model fields and independent
   textual row labels; color is reinforcement only.
3. Reserve Source and state columns before truncating titles in any dense list.
4. Make the wide Browser detail lens read-only. Resizing must not introduce a
   new focus target, key map, or navigation state.
5. Store a complete navigation snapshot per Destination: selection, scroll
   offset, active pane, and Destination-local query state.
6. Keep Search as a Destination with incremental grouped results and protected
   Source badges. `Enter` opens; `p` plays.
7. Keep the current item above a scrollable Queue. A Playlist starts its first
   available Track and queues the remainder; a Track clears the Queue; a Station
   is continuous and cannot be queued.
8. Replace, rather than squeeze, the frame below `80x24`; preserve state and keep
   resize recovery plus quit available.
9. Limit the contextual guide to about five valid actions. The second prototype
   guide row is evidence telemetry and need not ship as production copy.
10. Use the calm broadcast palette and sparse hierarchy, with one separator only
    where wide mode introduces the detail lens.

## Remaining questions

- Whether the wide split should begin at 100 columns, a different breakpoint,
  or from a content-width calculation; intermediate widths were not studied.
- Whether compact breadcrumbs should use middle ellipsis for unusually deep or
  long real names. The Browser preserved identity in this demo, but the final
  breadcrumb policy is unresolved.
- How long, localized, CJK, combining-character, or right-to-left catalog text
  affects the protected state/Source measurements.
- Contrast and legibility across real terminal themes and fonts beyond the
  deterministic RGB and `NO_COLOR` captures.
- Human-listener preference and speed. The prototype proves mechanical
  coherence, not that the hierarchy has passed usability testing.
- Real loading, playback advancement, errors, tuning motion, and terminal resize
  storms; all were deliberately excluded.
- Queue behavior when the current Track naturally advances, becomes
  unavailable, or is removed by an external playback adapter.

## Run and provenance

Run from the repository root:

```sh
cargo run --manifest-path prototype/persistent-frame/Cargo.toml
```

Deterministic walkthrough gate:

```sh
cargo run --manifest-path prototype/persistent-frame/Cargo.toml -- --verify-walkthroughs
```

Prototype code and screenshot commit:
`8e926d97a21961d924de3487e55fc77ee5ec9fb7`.

This is throwaway evidence. Preserve the branch and write production code from
the validated decisions; do not merge or promote this crate.
