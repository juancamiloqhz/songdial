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

Once the interaction model is validated, the next milestone will introduce a
small playback seam and connect one real playback adapter. CLIamp's current IPC
capabilities are the leading candidate. Additional Services should be added
only when their requirements demonstrate what genuinely varies.

The standalone application comes first. Omarchy integration should package and
surface a working Songdial experience rather than define its architecture.
