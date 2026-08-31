# Songdial product brief

## Vision

Songdial is the place where someone decides what to listen to, regardless of
which provider ultimately plays it.

It should make finding appropriate music feel immediate: choose a mood or
activity, browse a radio station, open a personal playlist, or search across
available sources without first navigating provider-specific menus.

## Problem

Music applications commonly expose their provider's catalog structure instead
of helping the listener answer a simpler question: _what should I play right
now?_

Terminal players compound the problem when they offer capable playback but
unclear navigation, limited discovery, provider-specific screens, or bundled
options that are difficult to understand and extend.

## Product promise

Songdial helps a listener reach suitable music quickly through a predictable,
keyboard-first interface that keeps browsing, playback, queue state, and source
details understandable at all times.

## Primary audience

Songdial is initially for developers and other keyboard-oriented users who:

- spend substantial time in a terminal;
- listen while working, studying, creating, or relaxing;
- use more than one kind of music source;
- value curated starting points without surrendering access to their own music;
- want a polished interface rather than a thin list of provider commands.

## Product principles

### Start from intent

Lead with the listener's intent—mood, activity, station, playlist, or search—not
with a provider selection screen.

### Keep navigation independent from playback

Opening an item reveals its contents or details. Playing it is a separate,
deliberate action. Starting playback must never trap the listener inside a
provider screen.

### Preserve context

The current selection, navigation path, playing item, queue, and source should
remain visible or immediately reachable.

### Curate without closing the system

Ship excellent starting options while allowing catalogs, stations, playlists,
and providers to grow without redesigning the whole experience.

### Make the terminal feel intentional

Use typography, color, layout, and motion to clarify state. Animation should
communicate tuning, loading, or transitions rather than decorate every action.

## Top-level experience

The home screen offers five paths:

1. **Mood & activity** — deep work, focus, flow, calm, energy, and other
   intention-oriented collections.
2. **Radio stations** — continuous curated or genre-oriented streams.
3. **My playlists** — personal and saved playlists from connected sources.
4. **Browse services** — provider catalogs when the listener explicitly wants
   them.
5. **Search everything** — one search entry point with source-aware results.

Now Playing and the queue are persistent application concerns rather than
destinations owned by any one provider.

## First milestone: Explore Songdial

The first executable milestone uses realistic demo data and simulated playback.
It must allow someone to:

1. launch Songdial into Home;
2. browse moods, stations, and playlists;
3. open a collection without playing it;
4. deliberately start simulated playback;
5. see persistent Now Playing and queue context;
6. search the demo catalog;
7. navigate backward predictably with `Esc`;
8. understand all essential keys without external documentation.

The milestone succeeds when the product's organization and keyboard behavior
feel coherent without relying on a real music provider to make the demo useful.

## Non-goals for the first milestone

- Real audio playback
- CLIamp integration
- YouTube Music or other account authentication
- A general-purpose provider plugin framework
- Omarchy installation or desktop widgets
- Catalog synchronization or persistence
- Recommendation algorithms
- Mobile, browser, or graphical desktop applications

## Direction after the first milestone

Once the interaction model is validated, the next milestone will introduce a
small playback seam and connect one real playback adapter. CLIamp's current IPC
capabilities are the leading candidate. Additional providers should be added
only when their requirements demonstrate what genuinely varies.

The standalone application comes first. Omarchy integration should package and
surface a working Songdial experience rather than define its architecture.
