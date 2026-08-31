# Handoff: persistent frame prototype

## Mission

Create a throwaway Ratatui prototype on a fresh `prototype/persistent-frame` branch and worktree. Answer one question:

> Does Songdial's fixed persistent frame preserve context without crowding the Browser at `80×24`, while a wide terminal adds useful detail without changing navigation?

The prototype is a primary source for a later `/to-spec` phase. Keep it out of `main`.

## Read first

- [`AGENTS.md`](../../AGENTS.md): repository agent pointers.
- [`docs/product.md`](../product.md): product promise, first milestone, and non-goals.
- [`docs/interaction-model.md`](../interaction-model.md): baseline observable interaction model.
- [`CONTEXT.md`](../../CONTEXT.md): canonical Songdial language.
- [`docs/adr/0001-build-the-first-milestone-as-a-durable-foundation.md`](../adr/0001-build-the-first-milestone-as-a-durable-foundation.md): why production work will outlive the prototype.
- [`docs/adr/0002-use-rust-and-ratatui.md`](../adr/0002-use-rust-and-ratatui.md): chosen application stack and trade-off.

The repository is <https://github.com/juancamiloqhz/songdial>. Start from the latest `origin/main` containing this handoff. This document contains the prototype-specific decisions resolved after the baseline interaction document; where they differ for this experiment, follow this document.

## Suggested skills

Call the Skill tool for:

1. **`prototype`** before building. Treat this as its UI branch, adapted to the user-mandated stateful Ratatui artifact rather than a browser route or web variant switcher.
2. **`frontend-design`** when translating the resolved visual direction into terminal hierarchy, color, typography, borders, and density.
3. **`handoff`** after committing and pushing the prototype, to return a portable verdict and pointers to the original task.

## Prototype contract

- Put the throwaway crate under `prototype/persistent-frame/` and mark it clearly as a prototype.
- Make it runnable with one command: `cargo run --manifest-path prototype/persistent-frame/Cargo.toml`.
- Keep all state in memory and use fixed fictional content.
- Build only the states and transitions needed by the scenario matrix below. Favor direct code over production abstractions, tests, persistence, networking, audio, configuration, or a generalized Service framework.
- Preserve the prototype on its branch. Production code will later be written from the findings rather than promoted from this crate.

## Resolved decision delta

### Frame and responsive behavior

- At `80×24`, stack four persistent regions vertically: one-line Location/breadcrumb, flexible Browser, two-line Now Playing, and a one- or two-line contextual key guide.
- The guide shows no more than about five context-valid actions. `?` may expose the full key set if needed by the walkthrough.
- At wide sizes, keep the persistent frame in the same order and add a details pane inside the Browser. Resizing changes information density, not navigation or state.
- Below `80×24`, replace the frame with a minimum-size message that reports current and required dimensions. Resizing back restores the exact prior state.

### Navigation and visible state

- Home order: Mood & activity, Radio stations, My playlists, Browse services, Search everything. Initial selection is Mood & activity.
- `Enter` opens without playing. `Esc` restores the previous Destination's exact selection, scroll offset, and active pane.
- Selection and playback are independent and must remain unmistakable without relying on color or animation.
- Search is a dedicated Destination. `/` focuses its query; results update while typing and group Listening intents, Stations, Playlists, and Tracks. Source badges are visible in Search.
- Now Playing persists while browsing. Its expanded Destination shows the current item first and a scrollable Queue below it.

### Playback and queue behavior needed for the walkthrough

- `p` deliberately starts a new Playback session. A Track starts with an empty Queue; a Playlist starts its first available Track and queues the remainder; a Station is continuous and uses an empty Queue.
- `a` appends a Track or a Playlist's Tracks. A Station cannot be queued.
- `Space` toggles playing and paused. Stations show `LIVE`; Tracks show deterministic progress.
- Queue entries can be selected, opened, played, and removed with `d`.
- Use textual markers and labels alongside color so Now Playing, paused/stopped state, selection, and unavailability remain legible under `NO_COLOR`.

### Visual direction

Songdial should feel like a calm broadcast instrument rather than a generic dashboard or neon hacker interface: charcoal background, warm neutral text, restrained amber for selection/tuning, signal green for active playback, muted Source badges, and borders only where they clarify hierarchy. Any tuning treatment is subtle and input remains immediate.

## Fixed prototype data

Use only enough fixed content to make density and state differences real:

- Six Listening intents, including Deep Work, Calm, Energy, and Reset.
- One populated intent path with enough Stations and Playlists to scroll at `80×24`.
- Two fictional Services and source-scoped results, including an apparent cross-service duplicate.
- One unavailable item with an explicit reason.
- A Playlist long enough to produce a Queue that scrolls.
- A Search query such as `night` that returns multiple concrete types and Sources.

Names should be plausible, concise, and readable in a narrow terminal. Avoid real brands and parody placeholders.

## Scenario and screenshot matrix

Capture rendered terminal screenshots at the exact cell sizes. Store them under `prototype/persistent-frame/screenshots/` with these names:

| File | Size | Required state |
| --- | --- | --- |
| `80x24-home.png` | `80×24` | Home, initial selection, no playback |
| `80x24-intent-playing.png` | `80×24` | Intent Browser scrolled, one item selected, a different item playing |
| `80x24-search.png` | `80×24` | Query and grouped multi-Source results with Now Playing preserved |
| `80x24-queue.png` | `80×24` | Expanded Now Playing and a Queue long enough to scroll |
| `80x24-intent-no-color.png` | `80×24` | Selection versus playback remains clear with `NO_COLOR` |
| `120x40-home.png` | `120×40` | Home with wide Browser detail treatment |
| `120x40-intent-playing.png` | `120×40` | Same logical intent/playback state as the compact capture, with details pane |
| `120x40-search.png` | `120×40` | Same logical Search state as the compact capture, with details pane |
| `120x40-queue.png` | `120×40` | Same logical Now Playing/Queue state as the compact capture |
| `79x23-too-small.png` | `79×23` | Minimum-size guard with quitting and resize recovery available |

Use the same logical state for paired compact/wide captures so density, hierarchy, and truncation can be compared directly.

## Required walkthroughs

Demonstrate these sequences in the running prototype:

1. Home → Listening intent → Playlist details → `p`; browse elsewhere while Now Playing persists.
2. Navigate back with `Esc` and verify the exact prior selection and scroll offset.
3. Open Search with `/`, enter a query, move through grouped results, and open one without triggering playback.
4. Open Now Playing with `n`, scroll the Queue, remove an entry with `d`, and return.
5. Resize an unchanged state between `80×24`, `120×40`, below-minimum, and back.

## Pass/fail gate

The prototype passes only when every criterion is evidenced by the walkthrough and named screenshots:

- Location, Browser, Now Playing, and key guidance never overlap at `80×24`.
- Essential Browser content remains usable; truncation never hides the selected state, playing state, or necessary Source disambiguation.
- Selection and playback are distinct with color and under `NO_COLOR`.
- Home, one Listening intent path, Search, and Now Playing/Queue are keyboard-usable at both target sizes.
- `Esc` restores the exact prior selection and scroll offset.
- Resizing preserves application state; the wide layout adds detail without changing navigation.
- The minimum-size guard recovers to the unchanged application state.

Any failed criterion means the answer is **fail**. Iterate the prototype and recapture the affected evidence before declaring pass.

## Findings and branch deliverables

Create `prototype/persistent-frame/FINDINGS.md` with:

1. The prototype question and a one-word verdict: `PASS` or `FAIL`.
2. Evidence for every pass/fail criterion, linked to screenshot filenames and walkthrough observations.
3. The compact and wide frame measurements that worked, including truncation or density choices.
4. Decisions ready to carry into `/to-spec`.
5. Remaining questions that the prototype could not answer.
6. The run command and prototype commit hash.

Commit the runnable crate, screenshots, and findings on `prototype/persistent-frame`, then push that branch. Do not merge it into `main`.

## Return to the original task

After pushing:

1. Call the `handoff` skill to create a portable return note containing the verdict, prototype branch and commit, and pointers to `FINDINGS.md` and the screenshots. Do not restate repository documents.
2. In the final response, give the user the branch name, commit hash, verdict, findings path, and the return-note path.
3. Ask the user to return to the original Songdial task and send:

   > The persistent-frame prototype is complete on `prototype/persistent-frame` at `<commit>`. Read `prototype/persistent-frame/FINDINGS.md` and the prototype return handoff, fold validated decisions back into the main design context, and continue toward `/to-spec`. Do not merge the prototype branch.
