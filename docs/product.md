# Songdial product brief

## Vision

Songdial is the place where a Listener decides what to hear, regardless of
which Service ultimately supplies it.

It should make finding appropriate music feel immediate: choose a Listening
intent, browse a Station, open a personal Playlist, or search across available
Sources without first navigating Service-specific menus.

## Problem

Music applications commonly expose their Service's catalog structure instead
of helping the Listener answer a simpler question: _what should I play right
now?_

Terminal players compound the problem when they offer capable playback but
unclear navigation, limited discovery, Service-specific Destinations, or bundled
options that are difficult to understand and extend.

## Product promise

Songdial helps a Listener reach suitable music quickly through a predictable,
keyboard-first interface that keeps browsing, the Playback session, Queue
state, and Source details understandable at all times.

## Primary audience

Songdial is initially for developers and other keyboard-oriented users who:

- spend substantial time in a terminal;
- listen while working, studying, creating, or relaxing;
- use more than one Service or Source;
- value curated starting points without surrendering access to their own music;
- want a polished interface rather than a thin list of Service commands.

## Product principles

### Start from intent

Lead with the Listener's intent—not with a Service selection Destination.

### Keep navigation independent from playback

Opening an item reveals its contents or details. Playing it is a separate,
deliberate action. Starting playback must never trap the Listener inside a
Service Destination.

### Preserve context

The current selection, navigation path, playing item, Queue, and Source should
remain visible or immediately reachable.

### Curate without closing the system

Ship excellent starting options while allowing catalogs, Stations, Playlists,
and Services to grow without redesigning the whole experience.

### Make the terminal feel intentional

Use typography, color, layout, and motion to clarify state. Animation should
communicate tuning, loading, or transitions rather than decorate every action.

## Top-level experience

The Home Destination offers five paths:

1. **Mood & activity** — deep work, focus, flow, calm, energy, and other
   Listening intents.
2. **Radio stations** — continuous curated or genre-oriented Stations.
3. **My playlists** — personal and saved Playlists from connected Sources.
4. **Browse services** — Service catalogs when the Listener explicitly wants
   them.
5. **Search everything** — one search entry point with Source-aware results.

Now Playing and the Queue are persistent application concerns rather than
Destinations owned by any one Service.

## First milestone: Explore Songdial

The first executable milestone uses the fixed Demo catalog and simulated
playback. It must allow a Listener to:

1. launch Songdial into Home;
2. browse Listening intents, Stations, and Playlists;
3. open a Destination or Playable item without playing it;
4. deliberately start simulated playback;
5. see persistent Now Playing and Queue context;
6. search the Demo catalog;
7. navigate backward predictably with `Esc`;
8. understand all essential keys without external documentation.

The milestone succeeds when the product's organization and keyboard behavior
feel coherent without relying on a real Service to make the demo useful.

## Non-goals for the first milestone

- Real audio playback
- CLIamp integration
- YouTube Music or other account authentication
- A general-purpose Service integration framework
- Omarchy installation or desktop widgets
- Catalog synchronization or persistence
- Recommendation algorithms
- Mobile, browser, or graphical desktop applications
- Mouse interaction
- Favorites or listening history
- Queue reordering, saving, or bulk editing
- Configuration files or themes
- Real network requests
- Cross-Service Track deduplication
- Support below an `80×24` terminal

## Direction after the first milestone

The next milestone will introduce a small playback seam and connect one real
playback adapter. Its first useful build will play curated public Stations and
a small catalog of real Tracks and Playlists without requiring accounts. The
catalog entries must describe the actual audio and its Source. Normal browsing
will show only real music; the fictional Demo catalog remains available for
explicit demo or testing use.

The initial selection emphasizes instrumental music for focus, calm, and gentle
energy. Finite Tracks load on demand over the network; their catalog metadata
and attribution ship with the application, while the recordings do not.

The real catalog's Home destination is named **Playlists**, reflecting its
curated listening sequences rather than implying a personal library.
**Browse sources** replaces Browse services and groups the real music by its
actual Source. The historical Explore Songdial model above retains its original
labels.

Real playback must work on all three existing packaged targets before the
milestone is complete: Linux x86-64, macOS arm64, and macOS x86-64. A successful
prototype on one platform does not satisfy that requirement.
Downloaded packages will include the required GStreamer runtime and plugins;
Listeners will not need to install GStreamer separately.

The replacement guarantees remain required: existing playback continues while
a replacement loads, failure preserves the existing Playback session and
Queue, and a superseded request must not later become audible. The application
commits the replacement item and Queue together after successful preparation.

The 2026-09-09 [CLIamp research](research/cliamp-playback-adapter.md) found that
its released headless behavior does not uphold those guarantees. Following
[alternative research](research/real-playback-adapter-alternatives.md) and the
[controlled-output prototype](handoffs/prototype-gstreamer-output.md), GStreamer
is selected for the first real playback adapter. The adapter will control when
prepared candidate audio reaches the output. Real audio, supported codecs,
and installed builds on all three targets remain implementation acceptance
requirements. See [the adapter decision](adr/0003-use-gstreamer-for-real-playback.md).
Additional Services should be added only when their requirements demonstrate
what genuinely varies.

Resuming a paused Station reconnects to the live broadcast. If a current Track
fails after starting, playback stops while retaining the Track and Queue, and
the application shows an error with a retry action. It does not automatically
skip that failed Track. If an already-playing Station disconnects, the
application briefly attempts automatic reconnection, then stops with a retry
action if recovery fails. Initial preparation has a 15-second deadline. Station
recovery allows at most three attempts within 30 seconds, with 1- and 3-second
delays between attempts; each attempt also respects the preparation deadline
and remaining recovery window. Pause, entering a stopped state, replacement
success, and quit cancel recovery. Timeout constants may be refined with recorded
implementation evidence without weakening bounded recovery or replacement
guarantees.

Track pause retains position; retry restarts a failed Track from the beginning
with its Queue intact. During a pending deliberate replacement, the previous
session continues normal Queue progression. Failure preserves that progressed
session and intervening Queue edits. Success invalidates its continuations and
commits the new item and Queue together. Finite completion waits for output to
drain; an error or materially incomplete recording retains the Track and Queue
with retry. Rounded catalog durations alone must not cause false failures.

If the next queued Track cannot load after the previous Track completes, it
becomes the retained failed current Track. The remaining Queue stays intact;
Retry attempts that failed Track from the beginning without skipping it.

When playback is requested but output has run out of decoded audio, 15 seconds
without more playable data triggers the existing Track failure or Station
recovery behavior, even if the connection remains open without an error or end
event. Decoded silence, deliberate pause, and normal completion do not count as
stalls. A Station's recovery window starts when the stall is declared. The stall
deadline may be tuned with recorded implementation evidence.

Explicit Retry supersedes a pending replacement: if failed Track A is retained
while B loads, retrying A cancels B and restarts A with A's Queue intact. B must
not later become audible. Ordinary pause/resume continues to affect only the
current session without canceling a pending replacement.

The approved initial selection is six Kevin MacLeod recordings and Radio
Paradise's Serenity and Mellow Mix Stations, documented in
[catalog research](research/initial-real-catalog.md). The Stations are not
promised to be instrumental-only. Songdial curates these ordered Playlists:

- **Focus:** Airport Lounge, Wholesome, Dream Culture.
- **Calm:** Music for Manatees, Water Prelude, Dream Culture.
- **Gentle energy:** Carefree, Airport Lounge, Wholesome.

Track details expose the artist, source link, and license credit; bundled credits
remain available offline. Source browsing groups recordings under Incompetech /
Kevin MacLeod and Stations under Radio Paradise. Playlist details distinguish
Songdial curation from recording provenance. Selection remains subject to actual
listening and decoding validation, especially Water Prelude's calm fit.

The Application owns navigation, the current item, Queue, and automatic
progression. A small playback interface hides decoding, controlled output,
network lifecycle, and cleanup. Most behavioral tests use the existing
Application event/effect boundary; focused adapter tests verify captured audio
and local HTTP failures. Packaged audible checks on all three targets complement
those deterministic tests. Playback observations continue below the supported
terminal size. Runtime and device errors are actionable, without silent Demo
fallback; quitting restores the terminal and stops owned audio with bounded
cleanup. Successful transitions need not be gapless.

The product owner will run or arrange actual listening checks on Linux x86-64,
macOS arm64, and macOS x86-64 when packages are ready. Required audible evidence
remains an acceptance gate for the affected tickets; arranging the checks does
not establish that they have passed.

The human validation study remains deferred until a packaged build can play
actual music and Stations. This sequencing does not satisfy Explore Songdial's
human-validation gate: [issue #11](https://github.com/juancamiloqhz/songdial/issues/11)
remains open, and [draft PR #22](https://github.com/juancamiloqhz/songdial/pull/22)
preserves its preparation. No participant observations or verdict are claimed.

The standalone application comes first. Omarchy integration should package and
surface a working Songdial experience rather than define its architecture.
