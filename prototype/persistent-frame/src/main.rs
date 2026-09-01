//! THROWAWAY PROTOTYPE. This crate is evidence for Songdial's persistent-frame
//! decision and is intentionally not production architecture.

use std::{
    env,
    error::Error,
    fs,
    io::{self, stdout},
    path::{Path, PathBuf},
    time::Duration,
};

use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use font8x8::{UnicodeFonts, BASIC_FONTS, BOX_FONTS, LATIN_FONTS, MISC_FONTS};
use image::{Rgb, RgbImage};
use ratatui::{
    backend::{CrosstermBackend, TestBackend},
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
    Frame, Terminal,
};

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;
const LOCATION_HEIGHT: u16 = 1;
const NOW_PLAYING_HEIGHT: u16 = 2;
const GUIDE_HEIGHT: u16 = 2;
const STATE_WIDTH: u16 = 11;
const SOURCE_WIDTH: u16 = 11;

const HOME_ITEMS: &[(&str, &str)] = &[
    (
        "Mood & activity",
        "Start from a Listening intent, before choosing a Source.",
    ),
    (
        "Radio stations",
        "Browse continuous Stations across Songdial and connected Services.",
    ),
    (
        "My playlists",
        "Open saved Playlists from both fictional Services.",
    ),
    (
        "Browse services",
        "Enter a Service catalog deliberately when provenance matters first.",
    ),
    (
        "Search everything",
        "Search grouped results without replacing the current Playback session.",
    ),
];

const INTENTS: &[(&str, &str)] = &[
    (
        "Deep Work",
        "Low-vocal, steady material for long concentration blocks.",
    ),
    (
        "Calm",
        "Soft edges and patient pacing for lowering intensity.",
    ),
    (
        "Energy",
        "Rhythmic, forward material for a deliberate lift.",
    ),
    (
        "Reset",
        "Short, clean sequences that mark a change of task.",
    ),
    (
        "Slow Morning",
        "Gentle starts with enough movement to become alert.",
    ),
    (
        "Late Night",
        "Quiet detail and low light for work after the room settles.",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Track {
    id: &'static str,
    title: &'static str,
    artist: &'static str,
    source: &'static str,
    seconds: u16,
    available: bool,
    unavailable_reason: Option<&'static str>,
}

const TRACKS: &[Track] = &[
    Track {
        id: "still-room",
        title: "Still Room",
        artist: "Vela Thread",
        source: "MORROW",
        seconds: 242,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "night-geometry-morrow",
        title: "Night Geometry",
        artist: "Vale & Pine",
        source: "MORROW",
        seconds: 260,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "desk-lamp",
        title: "Desk Lamp",
        artist: "Common Hours",
        source: "MORROW",
        seconds: 226,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "soft-machines",
        title: "Soft Machines",
        artist: "Kind Assembly",
        source: "MORROW",
        seconds: 251,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "north-window",
        title: "North Window",
        artist: "Vela Thread",
        source: "MORROW",
        seconds: 217,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "between-tasks",
        title: "Between Tasks",
        artist: "Paper Dial",
        source: "MORROW",
        seconds: 239,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "quiet-figures",
        title: "Quiet Figures",
        artist: "Common Hours",
        source: "MORROW",
        seconds: 248,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "last-tram",
        title: "Last Tram",
        artist: "Low Meridian",
        source: "MORROW",
        seconds: 271,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "pale-circuit",
        title: "Pale Circuit",
        artist: "Kind Assembly",
        source: "MORROW",
        seconds: 231,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "longhand",
        title: "Longhand",
        artist: "Paper Dial",
        source: "MORROW",
        seconds: 224,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "taped-window",
        title: "Taped Window",
        artist: "Vela Thread",
        source: "MORROW",
        seconds: 255,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "empty-platform",
        title: "Empty Platform",
        artist: "Low Meridian",
        source: "MORROW",
        seconds: 263,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "low-relay",
        title: "Low Relay",
        artist: "Common Hours",
        source: "MORROW",
        seconds: 220,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "static-bloom",
        title: "Static Bloom",
        artist: "Kind Assembly",
        source: "MORROW",
        seconds: 245,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "small-hours",
        title: "Small Hours",
        artist: "Paper Dial",
        source: "MORROW",
        seconds: 274,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "afterimage",
        title: "Afterimage",
        artist: "Vela Thread",
        source: "MORROW",
        seconds: 236,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "brass-rain",
        title: "Brass Rain",
        artist: "Low Meridian",
        source: "MORROW",
        seconds: 249,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "drafting-light",
        title: "Drafting Light",
        artist: "Common Hours",
        source: "MORROW",
        seconds: 228,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "slow-index",
        title: "Slow Index",
        artist: "Kind Assembly",
        source: "MORROW",
        seconds: 253,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "morning-buffer",
        title: "Morning Buffer",
        artist: "Paper Dial",
        source: "MORROW",
        seconds: 232,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "night-geometry-harbor",
        title: "Night Geometry",
        artist: "Vale & Pine",
        source: "HARBOR",
        seconds: 260,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "night-window",
        title: "Night Window",
        artist: "East Archive",
        source: "HARBOR",
        seconds: 244,
        available: true,
        unavailable_reason: None,
    },
    Track {
        id: "night-archive",
        title: "Night Archive",
        artist: "East Archive",
        source: "HARBOR",
        seconds: 238,
        available: false,
        unavailable_reason: Some("Harbor removed this recording from the demo region."),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Playlist {
    id: &'static str,
    title: &'static str,
    source: &'static str,
    description: &'static str,
    track_ids: &'static [&'static str],
}

const NIGHT_LEDGER_TRACKS: &[&str] = &[
    "still-room",
    "night-geometry-morrow",
    "desk-lamp",
    "soft-machines",
    "north-window",
    "between-tasks",
    "quiet-figures",
    "last-tram",
    "pale-circuit",
    "longhand",
    "taped-window",
    "empty-platform",
    "low-relay",
    "static-bloom",
    "small-hours",
    "afterimage",
    "brass-rain",
    "drafting-light",
    "slow-index",
    "morning-buffer",
];

const PLAYLISTS: &[Playlist] = &[
    Playlist {
        id: "quiet-hours",
        title: "Quiet Hours",
        source: "HARBOR",
        description: "Sparse instrumentals with wide gaps and soft attacks.",
        track_ids: &["night-window", "still-room", "north-window"],
    },
    Playlist {
        id: "open-loop",
        title: "Open Loop",
        source: "MORROW",
        description: "A measured sequence for work that should not announce itself.",
        track_ids: &["desk-lamp", "soft-machines", "between-tasks"],
    },
    Playlist {
        id: "terminal-garden",
        title: "Terminal Garden",
        source: "HARBOR",
        description: "Warm electronics and low-detail rhythm for coding sessions.",
        track_ids: &["night-geometry-harbor", "night-window", "quiet-figures"],
    },
    Playlist {
        id: "night-ledger",
        title: "Night Ledger",
        source: "MORROW",
        description: "Twenty steady Tracks for a long late shift.",
        track_ids: NIGHT_LEDGER_TRACKS,
    },
    Playlist {
        id: "night-ledger-harbor",
        title: "Night Ledger",
        source: "HARBOR",
        description: "Harbor's shorter Playlist with the same apparent title.",
        track_ids: &["night-geometry-harbor", "night-window", "night-archive"],
    },
    Playlist {
        id: "brass-and-rain",
        title: "Brass & Rain",
        source: "MORROW",
        description: "Muted brass, rain texture, and an even pulse.",
        track_ids: &["brass-rain", "low-relay", "small-hours"],
    },
    Playlist {
        id: "reset-cycle",
        title: "Reset Cycle",
        source: "HARBOR",
        description: "Three concise Tracks for changing tasks.",
        track_ids: &["night-window", "drafting-light", "morning-buffer"],
    },
    Playlist {
        id: "static-bloom-list",
        title: "Static Bloom",
        source: "MORROW",
        description: "Soft noise and harmonic repetition.",
        track_ids: &["static-bloom", "pale-circuit", "afterimage"],
    },
    Playlist {
        id: "daylight-circuit",
        title: "Daylight Circuit",
        source: "HARBOR",
        description: "Clearer edges for the final concentration block.",
        track_ids: &["morning-buffer", "drafting-light", "soft-machines"],
    },
    Playlist {
        id: "cadence-notes",
        title: "Cadence Notes",
        source: "MORROW",
        description: "A quiet sequence with a slightly firmer beat.",
        track_ids: &["longhand", "quiet-figures", "slow-index"],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Station {
    id: &'static str,
    title: &'static str,
    source: &'static str,
    style: &'static str,
    description: &'static str,
    available: bool,
    unavailable_reason: Option<&'static str>,
}

const STATIONS: &[Station] = &[
    Station {
        id: "low-orbit",
        title: "Low Orbit",
        source: "SONGDIAL",
        style: "ambient pulse",
        description: "A continuous low-motion signal curated by Songdial.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "paper-static",
        title: "Paper Static",
        source: "HARBOR",
        style: "textured ambient",
        description: "Dry texture and slow tonal movement.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "grainline",
        title: "Grainline",
        source: "MORROW",
        style: "minimal electronic",
        description: "Fine-grained electronics with an even floor.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "soft-focus",
        title: "Soft Focus",
        source: "SONGDIAL",
        style: "instrumental",
        description: "Soft-edged instrumental curation with no vocal center.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "night-signal",
        title: "Night Signal",
        source: "MORROW",
        style: "after-hours ambient",
        description: "A quiet after-hours stream with restrained movement.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "quiet-current",
        title: "Quiet Current",
        source: "HARBOR",
        style: "slow electronic",
        description: "A continuous current of low-density electronic music.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "slate-room",
        title: "Slate Room",
        source: "MORROW",
        style: "modern classical",
        description: "Piano, strings, and room tone without abrupt peaks.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "north-window-station",
        title: "North Window",
        source: "HARBOR",
        style: "acoustic ambient",
        description: "Acoustic figures with a cool, open sound.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "relay-field",
        title: "Relay Field",
        source: "SONGDIAL",
        style: "steady pulse",
        description: "Measured rhythm designed to stay behind the task.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "archive-88",
        title: "Archive 88",
        source: "MORROW",
        style: "archival broadcast",
        description: "A dormant archival stream retained to test unavailability.",
        available: false,
        unavailable_reason: Some("Morrow retired this Station; choose another Deep Work item."),
    },
    Station {
        id: "afterimage-station",
        title: "Afterimage",
        source: "HARBOR",
        style: "soft synthesis",
        description: "Long synth tails and a slow harmonic cycle.",
        available: true,
        unavailable_reason: None,
    },
    Station {
        id: "midnight-relay",
        title: "Midnight Relay",
        source: "HARBOR",
        style: "night broadcast",
        description: "A low-light continuous Station from Harbor.",
        available: true,
        unavailable_reason: None,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ItemRef {
    Track(&'static str),
    Playlist(&'static str),
    Station(&'static str),
}

const DEEP_WORK_ITEMS: &[ItemRef] = &[
    ItemRef::Station("low-orbit"),
    ItemRef::Playlist("quiet-hours"),
    ItemRef::Station("paper-static"),
    ItemRef::Playlist("open-loop"),
    ItemRef::Station("grainline"),
    ItemRef::Station("soft-focus"),
    ItemRef::Playlist("terminal-garden"),
    ItemRef::Station("night-signal"),
    ItemRef::Playlist("night-ledger"),
    ItemRef::Station("quiet-current"),
    ItemRef::Playlist("brass-and-rain"),
    ItemRef::Station("slate-room"),
    ItemRef::Playlist("reset-cycle"),
    ItemRef::Station("north-window-station"),
    ItemRef::Playlist("static-bloom-list"),
    ItemRef::Station("relay-field"),
    ItemRef::Playlist("daylight-circuit"),
    ItemRef::Station("archive-88"),
    ItemRef::Playlist("cadence-notes"),
    ItemRef::Station("afterimage-station"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchTarget {
    Intent(usize),
    Item(ItemRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SearchResult {
    group: &'static str,
    title: &'static str,
    subtitle: &'static str,
    source: &'static str,
    target: SearchTarget,
}

fn night_results() -> Vec<SearchResult> {
    vec![
        SearchResult {
            group: "LISTENING INTENTS",
            title: "Late Night",
            subtitle: "Listening intent",
            source: "SONGDIAL",
            target: SearchTarget::Intent(5),
        },
        SearchResult {
            group: "STATIONS",
            title: "Night Signal",
            subtitle: "Station",
            source: "MORROW",
            target: SearchTarget::Item(ItemRef::Station("night-signal")),
        },
        SearchResult {
            group: "STATIONS",
            title: "Midnight Relay",
            subtitle: "Station",
            source: "HARBOR",
            target: SearchTarget::Item(ItemRef::Station("midnight-relay")),
        },
        SearchResult {
            group: "PLAYLISTS",
            title: "Night Ledger",
            subtitle: "Playlist",
            source: "MORROW",
            target: SearchTarget::Item(ItemRef::Playlist("night-ledger")),
        },
        SearchResult {
            group: "PLAYLISTS",
            title: "Night Ledger",
            subtitle: "Playlist",
            source: "HARBOR",
            target: SearchTarget::Item(ItemRef::Playlist("night-ledger-harbor")),
        },
        SearchResult {
            group: "TRACKS",
            title: "Night Geometry - Vale & Pine",
            subtitle: "Track",
            source: "MORROW",
            target: SearchTarget::Item(ItemRef::Track("night-geometry-morrow")),
        },
        SearchResult {
            group: "TRACKS",
            title: "Night Geometry - Vale & Pine",
            subtitle: "Track",
            source: "HARBOR",
            target: SearchTarget::Item(ItemRef::Track("night-geometry-harbor")),
        },
        SearchResult {
            group: "TRACKS",
            title: "Night Archive - East Archive",
            subtitle: "Track - unavailable",
            source: "HARBOR",
            target: SearchTarget::Item(ItemRef::Track("night-archive")),
        },
    ]
}

fn track(id: &str) -> Track {
    *TRACKS
        .iter()
        .find(|item| item.id == id)
        .expect("fixed Track id")
}

fn playlist(id: &str) -> Playlist {
    *PLAYLISTS
        .iter()
        .find(|item| item.id == id)
        .expect("fixed Playlist id")
}

fn station(id: &str) -> Station {
    *STATIONS
        .iter()
        .find(|item| item.id == id)
        .expect("fixed Station id")
}

fn item_title(item: ItemRef) -> &'static str {
    match item {
        ItemRef::Track(id) => track(id).title,
        ItemRef::Playlist(id) => playlist(id).title,
        ItemRef::Station(id) => station(id).title,
    }
}

fn item_source(item: ItemRef) -> &'static str {
    match item {
        ItemRef::Track(id) => track(id).source,
        ItemRef::Playlist(id) => playlist(id).source,
        ItemRef::Station(id) => station(id).source,
    }
}

fn item_kind(item: ItemRef) -> &'static str {
    match item {
        ItemRef::Track(_) => "TRACK",
        ItemRef::Playlist(_) => "PLAYLIST",
        ItemRef::Station(_) => "STATION",
    }
}

fn item_available(item: ItemRef) -> bool {
    match item {
        ItemRef::Track(id) => track(id).available,
        ItemRef::Playlist(id) => playlist(id)
            .track_ids
            .iter()
            .any(|track_id| track(track_id).available),
        ItemRef::Station(id) => station(id).available,
    }
}

fn item_unavailable_reason(item: ItemRef) -> Option<&'static str> {
    match item {
        ItemRef::Track(id) => track(id).unavailable_reason,
        ItemRef::Playlist(_) => None,
        ItemRef::Station(id) => station(id).unavailable_reason,
    }
}

fn format_duration(seconds: u16) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Destination {
    Home,
    Intents,
    Intent(usize),
    Stations,
    Playlists,
    Services,
    Playlist(&'static str),
    Station(&'static str),
    Track(&'static str),
    Service(&'static str),
    Search,
    NowPlaying,
}

impl Destination {
    fn label(&self) -> String {
        match self {
            Destination::Home => "HOME".into(),
            Destination::Intents => "MOOD & ACTIVITY".into(),
            Destination::Intent(index) => INTENTS[*index].0.to_uppercase(),
            Destination::Stations => "RADIO STATIONS".into(),
            Destination::Playlists => "MY PLAYLISTS".into(),
            Destination::Services => "BROWSE SERVICES".into(),
            Destination::Playlist(id) => playlist(id).title.to_uppercase(),
            Destination::Station(id) => station(id).title.to_uppercase(),
            Destination::Track(id) => track(id).title.to_uppercase(),
            Destination::Service(name) => name.to_uppercase(),
            Destination::Search => "SEARCH EVERYTHING".into(),
            Destination::NowPlaying => "NOW PLAYING".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActivePane {
    List,
    Query,
    Queue,
    Detail,
}

impl ActivePane {
    fn label(self) -> &'static str {
        match self {
            ActivePane::List => "LIST",
            ActivePane::Query => "QUERY",
            ActivePane::Queue => "QUEUE",
            ActivePane::Detail => "DETAIL",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NavState {
    destination: Destination,
    selected: usize,
    scroll: usize,
    pane: ActivePane,
    query: String,
}

impl NavState {
    fn new(destination: Destination) -> Self {
        let pane = match destination {
            Destination::Search => ActivePane::Query,
            Destination::NowPlaying => ActivePane::Queue,
            Destination::Station(_) | Destination::Track(_) | Destination::Service(_) => {
                ActivePane::Detail
            }
            _ => ActivePane::List,
        };
        Self {
            destination,
            selected: 0,
            scroll: 0,
            pane,
            query: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayState {
    Playing,
    Paused,
    Live,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Playback {
    origin: ItemRef,
    current: ItemRef,
    state: PlayState,
    queue: Vec<&'static str>,
    progress_seconds: u16,
}

#[derive(Clone, Debug)]
struct App {
    current: NavState,
    back_stack: Vec<NavState>,
    playback: Option<Playback>,
    show_help: bool,
    no_color: bool,
    width: u16,
    height: u16,
    notice: String,
}

impl App {
    fn new(no_color: bool) -> Self {
        Self {
            current: NavState::new(Destination::Home),
            back_stack: Vec::new(),
            playback: None,
            show_help: false,
            no_color,
            width: MIN_WIDTH,
            height: MIN_HEIGHT,
            notice: "Opening never starts playback.".into(),
        }
    }

    fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    fn is_too_small(&self) -> bool {
        self.width < MIN_WIDTH || self.height < MIN_HEIGHT
    }

    fn breadcrumb(&self) -> String {
        let mut labels = vec!["SONGDIAL".to_string()];
        for state in self.back_stack.iter().chain(std::iter::once(&self.current)) {
            let label = state.destination.label();
            if labels.last() != Some(&label) {
                labels.push(label);
            }
        }
        labels.join(" / ")
    }

    fn state_fingerprint(&self) -> String {
        format!(
            "current={:?};back={:?};playback={:?};help={}",
            self.current, self.back_stack, self.playback, self.show_help
        )
    }

    fn open_destination(&mut self, destination: Destination) {
        let previous = self.current.clone();
        self.back_stack.push(previous);
        self.current = NavState::new(destination);
        self.show_help = false;
    }

    fn go_back(&mut self) {
        if let Some(previous) = self.back_stack.pop() {
            self.current = previous;
            self.notice = "Previous selection, offset, and pane restored.".into();
        } else {
            self.notice = "Home is the first Destination.".into();
        }
    }

    fn search_results(&self) -> Vec<SearchResult> {
        if self.current.query.to_lowercase().contains("night") {
            night_results()
        } else {
            Vec::new()
        }
    }

    fn selection_count(&self) -> usize {
        match &self.current.destination {
            Destination::Home => HOME_ITEMS.len(),
            Destination::Intents => INTENTS.len(),
            Destination::Intent(0) => DEEP_WORK_ITEMS.len(),
            Destination::Intent(_) => 0,
            Destination::Stations => STATIONS.len(),
            Destination::Playlists => PLAYLISTS.len(),
            Destination::Services => 2,
            Destination::Playlist(id) => playlist(id).track_ids.len(),
            Destination::Search => self.search_results().len(),
            Destination::NowPlaying => self
                .playback
                .as_ref()
                .map(|playback| playback.queue.len())
                .unwrap_or(0),
            Destination::Station(_) | Destination::Track(_) | Destination::Service(_) => 0,
        }
    }

    fn browser_height(&self) -> usize {
        self.height
            .saturating_sub(LOCATION_HEIGHT + NOW_PLAYING_HEIGHT + GUIDE_HEIGHT) as usize
    }

    fn list_capacity(&self) -> usize {
        let browser = self.browser_height();
        match self.current.destination {
            Destination::Home => browser.saturating_sub(3),
            Destination::Intents => browser.saturating_sub(3),
            Destination::Intent(_) => browser.saturating_sub(3),
            Destination::Stations | Destination::Playlists | Destination::Services => {
                browser.saturating_sub(3)
            }
            Destination::Playlist(_) => browser.saturating_sub(4),
            Destination::Search => browser.saturating_sub(3),
            Destination::NowPlaying => browser.saturating_sub(5),
            Destination::Station(_) | Destination::Track(_) | Destination::Service(_) => browser,
        }
        .max(1)
    }

    fn ensure_selected_visible(&mut self) {
        let count = self.selection_count();
        if count == 0 {
            self.current.selected = 0;
            self.current.scroll = 0;
            return;
        }
        self.current.selected = self.current.selected.min(count - 1);
        let capacity = self.list_capacity();
        if self.current.selected < self.current.scroll {
            self.current.scroll = self.current.selected;
        } else if self.current.selected >= self.current.scroll + capacity {
            self.current.scroll = self.current.selected + 1 - capacity;
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if matches!(self.current.destination, Destination::Search)
            && self.current.pane == ActivePane::Query
        {
            if delta > 0 && !self.search_results().is_empty() {
                self.current.pane = ActivePane::List;
                self.current.selected = 0;
                self.notice = "Results focused; opening remains separate from playback.".into();
            }
            return;
        }

        let count = self.selection_count();
        if count == 0 {
            self.notice = "Nothing selectable in this focused prototype state.".into();
            return;
        }
        if delta < 0 && self.current.selected == 0 {
            if matches!(self.current.destination, Destination::Search) {
                self.current.pane = ActivePane::Query;
                self.notice = "Query focused.".into();
            }
            return;
        }
        let next = (self.current.selected as i32 + delta).clamp(0, count as i32 - 1);
        self.current.selected = next as usize;
        self.ensure_selected_visible();
        self.notice = format!(
            "Selection {} of {}; Playback did not change.",
            self.current.selected + 1,
            count
        );
    }

    fn selected_item(&self) -> Option<ItemRef> {
        match &self.current.destination {
            Destination::Intent(0) => DEEP_WORK_ITEMS.get(self.current.selected).copied(),
            Destination::Stations => STATIONS
                .get(self.current.selected)
                .map(|item| ItemRef::Station(item.id)),
            Destination::Playlists => PLAYLISTS
                .get(self.current.selected)
                .map(|item| ItemRef::Playlist(item.id)),
            Destination::Playlist(id) => playlist(id)
                .track_ids
                .get(self.current.selected)
                .copied()
                .map(ItemRef::Track),
            Destination::Station(id) => Some(ItemRef::Station(id)),
            Destination::Track(id) => Some(ItemRef::Track(id)),
            Destination::Search => {
                self.search_results()
                    .get(self.current.selected)
                    .and_then(|result| match result.target {
                        SearchTarget::Intent(_) => None,
                        SearchTarget::Item(item) => Some(item),
                    })
            }
            Destination::NowPlaying => self
                .playback
                .as_ref()
                .and_then(|playback| playback.queue.get(self.current.selected))
                .copied()
                .map(ItemRef::Track),
            _ => None,
        }
    }

    fn open_selected(&mut self) {
        match self.current.destination.clone() {
            Destination::Home => match self.current.selected {
                0 => self.open_destination(Destination::Intents),
                1 => self.open_destination(Destination::Stations),
                2 => self.open_destination(Destination::Playlists),
                3 => self.open_destination(Destination::Services),
                4 => self.open_destination(Destination::Search),
                _ => {}
            },
            Destination::Intents => {
                self.open_destination(Destination::Intent(self.current.selected));
            }
            Destination::Intent(0)
            | Destination::Stations
            | Destination::Playlists
            | Destination::Playlist(_)
            | Destination::NowPlaying => {
                if let Some(item) = self.selected_item() {
                    let destination = match item {
                        ItemRef::Track(id) => Destination::Track(id),
                        ItemRef::Playlist(id) => Destination::Playlist(id),
                        ItemRef::Station(id) => Destination::Station(id),
                    };
                    self.open_destination(destination);
                }
            }
            Destination::Intent(_) => {
                self.notice = "Only Deep Work is populated for this focused prototype.".into();
            }
            Destination::Services => {
                let name = if self.current.selected == 0 {
                    "Morrow"
                } else {
                    "Harbor"
                };
                self.open_destination(Destination::Service(name));
            }
            Destination::Search => {
                if self.current.pane == ActivePane::Query {
                    if self.search_results().is_empty() {
                        self.notice = "Type night to populate the fixed demo results.".into();
                    } else {
                        self.current.pane = ActivePane::List;
                        self.notice = "Results focused; press Enter again to open.".into();
                    }
                    return;
                }
                if let Some(result) = self.search_results().get(self.current.selected).copied() {
                    match result.target {
                        SearchTarget::Intent(index) => {
                            self.open_destination(Destination::Intent(index))
                        }
                        SearchTarget::Item(ItemRef::Track(id)) => {
                            self.open_destination(Destination::Track(id))
                        }
                        SearchTarget::Item(ItemRef::Playlist(id)) => {
                            self.open_destination(Destination::Playlist(id))
                        }
                        SearchTarget::Item(ItemRef::Station(id)) => {
                            self.open_destination(Destination::Station(id))
                        }
                    }
                }
            }
            Destination::Station(_) | Destination::Track(_) | Destination::Service(_) => {
                self.notice = "This Destination is already open; p starts playback.".into();
            }
        }
    }

    fn start_playback(&mut self, item: ItemRef) {
        if !item_available(item) {
            self.notice = item_unavailable_reason(item)
                .unwrap_or("This item is unavailable.")
                .into();
            return;
        }

        let playback = match item {
            ItemRef::Track(id) => Playback {
                origin: item,
                current: item,
                state: PlayState::Playing,
                queue: Vec::new(),
                progress_seconds: 134.min(track(id).seconds),
            },
            ItemRef::Station(_) => Playback {
                origin: item,
                current: item,
                state: PlayState::Live,
                queue: Vec::new(),
                progress_seconds: 0,
            },
            ItemRef::Playlist(id) => {
                let available: Vec<&'static str> = playlist(id)
                    .track_ids
                    .iter()
                    .copied()
                    .filter(|track_id| track(track_id).available)
                    .collect();
                let first = available[0];
                Playback {
                    origin: item,
                    current: ItemRef::Track(first),
                    state: PlayState::Playing,
                    queue: available.into_iter().skip(1).collect(),
                    progress_seconds: 134.min(track(first).seconds),
                }
            }
        };
        let queue_len = playback.queue.len();
        self.playback = Some(playback);
        self.notice = format!(
            "Started a new {} Playback session; Queue has {} Track(s).",
            item_kind(item),
            queue_len
        );
    }

    fn play_selected(&mut self) {
        let item = match self.current.destination {
            Destination::Playlist(id) => Some(ItemRef::Playlist(id)),
            _ => self.selected_item(),
        };
        if let Some(item) = item {
            self.start_playback(item);
            if matches!(self.current.destination, Destination::NowPlaying) {
                self.current.selected = 0;
                self.current.scroll = 0;
            }
        } else {
            self.notice = "The current selection is not a Playable item.".into();
        }
    }

    fn add_selected(&mut self) {
        let item = match self.current.destination {
            Destination::Playlist(id) => Some(ItemRef::Playlist(id)),
            _ => self.selected_item(),
        };
        let Some(item) = item else {
            self.notice = "The current selection cannot be added to the Queue.".into();
            return;
        };
        if matches!(item, ItemRef::Station(_)) {
            self.notice = "Stations are continuous and cannot be queued.".into();
            return;
        }
        if !item_available(item) {
            self.notice = item_unavailable_reason(item)
                .unwrap_or("This item is unavailable.")
                .into();
            return;
        }
        let Some(playback) = self.playback.as_mut() else {
            self.notice = "Start a Playback session with p before appending.".into();
            return;
        };
        let mut added = 0;
        match item {
            ItemRef::Track(id) => {
                playback.queue.push(id);
                added = 1;
            }
            ItemRef::Playlist(id) => {
                for track_id in playlist(id).track_ids.iter().copied() {
                    if track(track_id).available {
                        playback.queue.push(track_id);
                        added += 1;
                    }
                }
            }
            ItemRef::Station(_) => unreachable!(),
        }
        self.notice = format!("Appended {added} Track(s) to the Queue.");
    }

    fn toggle_playback(&mut self) {
        let Some(playback) = self.playback.as_mut() else {
            self.notice = "Nothing is playing; select a Playable item and press p.".into();
            return;
        };
        playback.state = match playback.state {
            PlayState::Playing | PlayState::Live => PlayState::Paused,
            PlayState::Paused => match playback.current {
                ItemRef::Station(_) => PlayState::Live,
                _ => PlayState::Playing,
            },
        };
        self.notice = match playback.state {
            PlayState::Playing => "Playback resumed.".into(),
            PlayState::Paused => "Playback paused.".into(),
            PlayState::Live => "Station resumed LIVE.".into(),
        };
    }

    fn remove_queue_entry(&mut self) {
        if !matches!(self.current.destination, Destination::NowPlaying) {
            self.notice = "d removes only the selected Queue entry.".into();
            return;
        }
        let Some(playback) = self.playback.as_mut() else {
            self.notice = "The Queue is empty.".into();
            return;
        };
        if playback.queue.is_empty() {
            self.notice = "The Queue is empty.".into();
            return;
        }
        let index = self.current.selected.min(playback.queue.len() - 1);
        let removed = playback.queue.remove(index);
        self.notice = format!("Removed {} from the Queue.", track(removed).title);
        self.ensure_selected_visible();
    }

    fn focus_search(&mut self) {
        if matches!(self.current.destination, Destination::Search) {
            self.current.pane = ActivePane::Query;
            self.notice = "Query focused.".into();
        } else {
            self.open_destination(Destination::Search);
            self.notice = "Search is a Destination; Now Playing remains unchanged.".into();
        }
    }

    fn open_now_playing(&mut self) {
        if matches!(self.current.destination, Destination::NowPlaying) {
            self.notice = "Queue is already open.".into();
        } else {
            self.open_destination(Destination::NowPlaying);
            self.notice = "Current item first; Queue below.".into();
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) -> Option<bool> {
        if !matches!(self.current.destination, Destination::Search)
            || self.current.pane != ActivePane::Query
        {
            return None;
        }
        match key.code {
            KeyCode::Esc => {
                self.go_back();
                Some(false)
            }
            KeyCode::Down => {
                self.move_selection(1);
                Some(false)
            }
            KeyCode::Enter => {
                self.open_selected();
                Some(false)
            }
            KeyCode::Backspace => {
                self.current.query.pop();
                self.current.selected = 0;
                self.current.scroll = 0;
                self.notice = "Results updated while typing.".into();
                Some(false)
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                Some(false)
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.current.query.push(character);
                self.current.selected = 0;
                self.current.scroll = 0;
                self.notice = "Results updated while typing.".into();
                Some(false)
            }
            _ => Some(false),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }
        if self.is_too_small() {
            return matches!(key.code, KeyCode::Char('q'));
        }
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => self.show_help = false,
                _ => {}
            }
            return false;
        }
        if let Some(quit) = self.handle_search_input(key) {
            return quit;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Enter => self.open_selected(),
            KeyCode::Esc => self.go_back(),
            KeyCode::Char('/') => self.focus_search(),
            KeyCode::Char('n') => self.open_now_playing(),
            KeyCode::Char('p') => self.play_selected(),
            KeyCode::Char('a') => self.add_selected(),
            KeyCode::Char('d') => self.remove_queue_entry(),
            KeyCode::Char(' ') => self.toggle_playback(),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('q') if matches!(self.current.destination, Destination::Home) => {
                return true;
            }
            KeyCode::Char('q') => {
                self.notice = "Press Esc to return; q quits from Home.".into();
            }
            _ => {}
        }
        false
    }
}

#[derive(Clone, Copy)]
struct Palette {
    no_color: bool,
}

impl Palette {
    const CHARCOAL: Color = Color::Rgb(16, 19, 21);
    const PAPER: Color = Color::Rgb(230, 221, 201);
    const AMBER: Color = Color::Rgb(214, 154, 58);
    const GREEN: Color = Color::Rgb(101, 185, 123);
    const BRASS: Color = Color::Rgb(133, 123, 105);
    const SLATE: Color = Color::Rgb(32, 38, 41);

    fn fg(self, color: Color) -> Color {
        if self.no_color {
            Color::Reset
        } else {
            color
        }
    }

    fn bg(self, color: Color) -> Color {
        if self.no_color {
            Color::Reset
        } else {
            color
        }
    }

    fn base(self) -> Style {
        Style::default()
            .fg(self.fg(Self::PAPER))
            .bg(self.bg(Self::CHARCOAL))
    }

    fn location(self) -> Style {
        self.base()
            .bg(self.bg(Self::SLATE))
            .add_modifier(Modifier::BOLD)
    }

    fn title(self) -> Style {
        self.base().add_modifier(Modifier::BOLD)
    }

    fn utility(self) -> Style {
        self.base()
            .fg(self.fg(Self::BRASS))
            .add_modifier(Modifier::DIM)
    }

    fn amber(self) -> Style {
        self.base()
            .fg(self.fg(Self::AMBER))
            .add_modifier(Modifier::BOLD)
    }

    fn green(self) -> Style {
        self.base()
            .fg(self.fg(Self::GREEN))
            .add_modifier(Modifier::BOLD)
    }

    fn selection(self) -> Style {
        if self.no_color {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            self.base()
                .fg(Self::AMBER)
                .bg(Self::SLATE)
                .add_modifier(Modifier::BOLD)
        }
    }

    fn playing(self) -> Style {
        if self.no_color {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            self.base().fg(Self::GREEN).add_modifier(Modifier::BOLD)
        }
    }

    fn unavailable(self) -> Style {
        self.base()
            .fg(self.fg(Self::BRASS))
            .add_modifier(Modifier::BOLD)
    }

    fn badge(self) -> Style {
        self.base()
            .fg(self.fg(Self::BRASS))
            .bg(self.bg(Self::SLATE))
            .add_modifier(Modifier::DIM)
    }
}

fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let palette = Palette {
        no_color: app.no_color,
    };
    frame.render_widget(Block::default().style(palette.base()), area);

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minimum_guard(frame, area, palette);
        return;
    }

    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(LOCATION_HEIGHT),
            Constraint::Min(1),
            Constraint::Length(NOW_PLAYING_HEIGHT),
            Constraint::Length(GUIDE_HEIGHT),
        ])
        .split(area);

    render_location(frame, regions[0], app, palette);
    if app.show_help {
        render_help(frame, regions[1], palette);
    } else {
        render_browser(frame, regions[1], app, palette);
    }
    render_now_playing(frame, regions[2], app, palette);
    render_guide(frame, regions[3], app, palette);
}

fn render_minimum_guard(frame: &mut Frame<'_>, area: Rect, palette: Palette) {
    let message = vec![
        Line::from(Span::styled("SONGDIAL NEEDS MORE ROOM", palette.title())),
        Line::from(""),
        Line::from(format!("Current: {}x{} cells", area.width, area.height)),
        Line::from(format!("Required: {}x{} cells", MIN_WIDTH, MIN_HEIGHT)),
        Line::from(""),
        Line::from("Resize to recover the unchanged session."),
        Line::from(Span::styled("q quit", palette.amber())),
    ];
    let height = message.len() as u16;
    let centered = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(height) / 2,
        area.width,
        height,
    );
    frame.render_widget(
        Paragraph::new(message)
            .style(palette.base())
            .alignment(Alignment::Center),
        centered,
    );
}

fn render_location(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" SONGDIAL ", palette.amber()),
            Span::styled(
                format!("/ {}", app.breadcrumb().trim_start_matches("SONGDIAL / ")),
                palette.location(),
            ),
        ]))
        .style(palette.location()),
        area,
    );
}

fn browser_areas(area: Rect) -> (Rect, Option<Rect>, Option<Rect>) {
    if area.width >= 100 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(48),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);
        (columns[0], Some(columns[1]), Some(columns[2]))
    } else {
        (area, None, None)
    }
}

fn line_area(area: Rect, row: u16) -> Option<Rect> {
    (row < area.height).then_some(Rect::new(area.x, area.y + row, area.width, 1))
}

fn render_text(frame: &mut Frame<'_>, area: Rect, text: impl Into<Line<'static>>, style: Style) {
    frame.render_widget(Paragraph::new(text.into()).style(style), area);
}

fn render_header(
    frame: &mut Frame<'_>,
    area: Rect,
    title: impl Into<String>,
    subtitle: impl Into<String>,
    palette: Palette,
) {
    if let Some(row) = line_area(area, 0) {
        render_text(frame, row, format!("  {}", title.into()), palette.title());
    }
    if let Some(row) = line_area(area, 1) {
        render_text(
            frame,
            row,
            format!("  {}", subtitle.into()),
            palette.utility(),
        );
    }
    if let Some(row) = line_area(area, 2) {
        render_text(
            frame,
            row,
            "  STATE RAIL: SELECTED / PLAYING / UNAVAIL     Source stays pinned right",
            palette.utility(),
        );
    }
}

fn playback_marks_item(app: &App, item: ItemRef) -> bool {
    app.playback
        .as_ref()
        .map(|playback| playback.origin == item || playback.current == item)
        .unwrap_or(false)
}

fn render_item_row(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App,
    palette: Palette,
    item: ItemRef,
    selected: bool,
) {
    let playing = playback_marks_item(app, item);
    let available = item_available(item);
    let row_style = if selected {
        palette.selection()
    } else {
        palette.base()
    };
    frame.render_widget(Block::default().style(row_style), area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(STATE_WIDTH),
            Constraint::Min(1),
            Constraint::Length(SOURCE_WIDTH),
        ])
        .split(area);

    let state_label = match (selected, playing, available) {
        (true, true, _) => "SEL+PLAY >",
        (true, false, false) => "SELECTED !",
        (true, false, true) => "SELECTED >",
        (false, true, _) => "PLAYING * ",
        (false, false, false) => "UNAVAIL ! ",
        (false, false, true) => "          ",
    };
    let state_style = if playing && !selected {
        palette.playing()
    } else if !available {
        palette.unavailable()
    } else if selected {
        palette.selection()
    } else {
        row_style
    };
    render_text(frame, columns[0], format!(" {state_label}"), state_style);

    let unavailable_prefix = if available { "" } else { "! " };
    render_text(
        frame,
        columns[1],
        format!(
            "{}{}  {}",
            unavailable_prefix,
            item_kind(item),
            item_title(item)
        ),
        if playing && !selected {
            palette.playing()
        } else {
            row_style
        },
    );

    frame.render_widget(
        Paragraph::new(format!("[{}]", item_source(item)))
            .style(palette.badge())
            .alignment(Alignment::Right),
        columns[2],
    );
}

fn render_named_row(
    frame: &mut Frame<'_>,
    area: Rect,
    palette: Palette,
    selected: bool,
    title: &str,
    kind: &str,
    source: &str,
    unavailable: bool,
) {
    let style = if selected {
        palette.selection()
    } else if unavailable {
        palette.unavailable()
    } else {
        palette.base()
    };
    frame.render_widget(Block::default().style(style), area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(STATE_WIDTH),
            Constraint::Min(1),
            Constraint::Length(SOURCE_WIDTH),
        ])
        .split(area);
    let state = if selected && unavailable {
        "SELECTED !"
    } else if selected {
        "SELECTED >"
    } else if unavailable {
        "UNAVAIL ! "
    } else {
        "          "
    };
    render_text(frame, columns[0], format!(" {state}"), style);
    render_text(frame, columns[1], format!("{kind}  {title}"), style);
    if !source.is_empty() {
        frame.render_widget(
            Paragraph::new(format!("[{source}]"))
                .style(palette.badge())
                .alignment(Alignment::Right),
            columns[2],
        );
    }
}

fn render_separator(frame: &mut Frame<'_>, area: Rect, palette: Palette) {
    for row in 0..area.height {
        if let Some(row_area) = line_area(area, row) {
            render_text(frame, row_area, "|", palette.utility());
        }
    }
}

fn render_browser(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    let (list_area, separator, details_area) = browser_areas(area);
    match &app.current.destination {
        Destination::Home => render_home(frame, list_area, app, palette),
        Destination::Intents => render_intents(frame, list_area, app, palette),
        Destination::Intent(index) => render_intent(frame, list_area, app, palette, *index),
        Destination::Stations => render_stations(frame, list_area, app, palette),
        Destination::Playlists => render_playlists(frame, list_area, app, palette),
        Destination::Services => render_services(frame, list_area, app, palette),
        Destination::Playlist(id) => render_playlist(frame, list_area, app, palette, id),
        Destination::Station(id) => render_station(frame, list_area, palette, id),
        Destination::Track(id) => render_track(frame, list_area, palette, id),
        Destination::Service(name) => render_service(frame, list_area, palette, name),
        Destination::Search => render_search(frame, list_area, app, palette),
        Destination::NowPlaying => render_queue(frame, list_area, app, palette),
    }
    if let Some(separator) = separator {
        render_separator(frame, separator, palette);
    }
    if let Some(details_area) = details_area {
        render_details(frame, details_area, app, palette);
    }
}

fn render_home(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    render_header(
        frame,
        area,
        "CHOOSE HOW TO LISTEN",
        "Intent first; Source appears only where it clarifies a choice.",
        palette,
    );
    for (visible_index, (title, _)) in HOME_ITEMS.iter().enumerate() {
        let row = 4 + visible_index as u16;
        let Some(row_area) = line_area(area, row) else {
            break;
        };
        render_named_row(
            frame,
            row_area,
            palette,
            app.current.selected == visible_index,
            title,
            "DEST",
            "",
            false,
        );
    }
}

fn render_intents(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    render_header(
        frame,
        area,
        "MOOD & ACTIVITY",
        "Six Listening intents; Deep Work is populated for this focused prototype.",
        palette,
    );
    for (index, (title, _)) in INTENTS.iter().enumerate() {
        let Some(row_area) = line_area(area, 3 + index as u16) else {
            break;
        };
        render_named_row(
            frame,
            row_area,
            palette,
            app.current.selected == index,
            title,
            "INTENT",
            "",
            false,
        );
    }
}

fn render_intent(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette, index: usize) {
    let (title, description) = INTENTS[index];
    render_header(frame, area, title.to_uppercase(), description, palette);
    if index != 0 {
        let message = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  This Listening intent is intentionally empty.",
                palette.title(),
            )),
            Line::from("  Deep Work carries the prototype's populated path."),
            Line::from("  Press Esc to return and open Deep Work."),
        ];
        if area.height > 4 {
            frame.render_widget(
                Paragraph::new(message).style(palette.base()),
                Rect::new(area.x, area.y + 3, area.width, area.height - 3),
            );
        }
        return;
    }

    let start = app.current.scroll;
    for (visible, item) in DEEP_WORK_ITEMS.iter().copied().enumerate().skip(start) {
        let display_row = visible - start;
        let Some(row_area) = line_area(area, 3 + display_row as u16) else {
            break;
        };
        render_item_row(
            frame,
            row_area,
            app,
            palette,
            item,
            app.current.selected == visible,
        );
    }
}

fn render_stations(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    render_header(
        frame,
        area,
        "RADIO STATIONS",
        "Continuous Stations start with p and always use an empty Queue.",
        palette,
    );
    let start = app.current.scroll;
    for (index, item) in STATIONS.iter().enumerate().skip(start) {
        let Some(row_area) = line_area(area, 3 + (index - start) as u16) else {
            break;
        };
        render_item_row(
            frame,
            row_area,
            app,
            palette,
            ItemRef::Station(item.id),
            app.current.selected == index,
        );
    }
}

fn render_playlists(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    render_header(
        frame,
        area,
        "MY PLAYLISTS",
        "Opening inspects a Playlist; p deliberately starts it.",
        palette,
    );
    let start = app.current.scroll;
    for (index, item) in PLAYLISTS.iter().enumerate().skip(start) {
        let Some(row_area) = line_area(area, 3 + (index - start) as u16) else {
            break;
        };
        render_item_row(
            frame,
            row_area,
            app,
            palette,
            ItemRef::Playlist(item.id),
            app.current.selected == index,
        );
    }
}

fn render_services(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    render_header(
        frame,
        area,
        "BROWSE SERVICES",
        "A deliberate Source-first entry point; Songdial navigation still owns Esc.",
        palette,
    );
    let services = [
        (
            "Morrow",
            "Fictional Service with the long Night Ledger Playlist.",
        ),
        (
            "Harbor",
            "Fictional Service with a Source-scoped duplicate.",
        ),
    ];
    for (index, (title, _)) in services.iter().enumerate() {
        let Some(row_area) = line_area(area, 4 + index as u16) else {
            break;
        };
        render_named_row(
            frame,
            row_area,
            palette,
            app.current.selected == index,
            title,
            "SERVICE",
            &title.to_uppercase(),
            false,
        );
    }
}

fn render_playlist(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette, id: &str) {
    let playlist = playlist(id);
    if let Some(row) = line_area(area, 0) {
        render_text(
            frame,
            row,
            format!(
                "  PLAYLIST  {}  [{}]",
                playlist.title.to_uppercase(),
                playlist.source
            ),
            palette.title(),
        );
    }
    if let Some(row) = line_area(area, 1) {
        render_text(
            frame,
            row,
            format!("  {}", playlist.description),
            palette.utility(),
        );
    }
    if let Some(row) = line_area(area, 2) {
        render_text(
            frame,
            row,
            "  p starts this Playlist | a appends it | Enter inspects selected Track",
            palette.amber(),
        );
    }
    if let Some(row) = line_area(area, 3) {
        render_text(
            frame,
            row,
            format!("  {} TRACKS", playlist.track_ids.len()),
            palette.utility(),
        );
    }
    let start = app.current.scroll;
    for (index, track_id) in playlist.track_ids.iter().copied().enumerate().skip(start) {
        let Some(row_area) = line_area(area, 4 + (index - start) as u16) else {
            break;
        };
        render_item_row(
            frame,
            row_area,
            app,
            palette,
            ItemRef::Track(track_id),
            app.current.selected == index,
        );
    }
}

fn render_station(frame: &mut Frame<'_>, area: Rect, palette: Palette, id: &str) {
    let station = station(id);
    let availability = if station.available {
        "AVAILABLE - p starts a LIVE Station with an empty Queue"
    } else {
        "UNAVAILABLE"
    };
    let mut lines = vec![
        Line::from(Span::styled("  STATION", palette.utility())),
        Line::from(Span::styled(
            format!("  {}", station.title),
            palette.title(),
        )),
        Line::from(format!("  Source [{}]", station.source)),
        Line::from(format!("  Style  {}", station.style)),
        Line::from(""),
        Line::from(format!("  {}", station.description)),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {availability}"),
            if station.available {
                palette.green()
            } else {
                palette.unavailable()
            },
        )),
    ];
    if let Some(reason) = station.unavailable_reason {
        lines.push(Line::from(format!("  Reason: {reason}")));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .style(palette.base())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_track(frame: &mut Frame<'_>, area: Rect, palette: Palette, id: &str) {
    let track = track(id);
    let availability = if track.available {
        "AVAILABLE"
    } else {
        "UNAVAILABLE"
    };
    let mut lines = vec![
        Line::from(Span::styled("  TRACK", palette.utility())),
        Line::from(Span::styled(format!("  {}", track.title), palette.title())),
        Line::from(format!("  {}", track.artist)),
        Line::from(format!("  Source [{}]", track.source)),
        Line::from(format!("  Duration {}", format_duration(track.seconds))),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {availability}"),
            if track.available {
                palette.green()
            } else {
                palette.unavailable()
            },
        )),
        Line::from("  Opening this Track did not change playback; press p deliberately."),
    ];
    if let Some(reason) = track.unavailable_reason {
        lines.push(Line::from(format!("  Reason: {reason}")));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .style(palette.base())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_service(frame: &mut Frame<'_>, area: Rect, palette: Palette, name: &str) {
    let description = if name == "Morrow" {
        "Morrow exposes the long Night Ledger Playlist and the first Night Geometry Track."
    } else {
        "Harbor exposes an apparent Night Ledger and Night Geometry duplicate; Source remains explicit."
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("  FICTIONAL SERVICE", palette.utility())),
            Line::from(Span::styled(format!("  {name}"), palette.title())),
            Line::from(""),
            Line::from(format!("  {description}")),
            Line::from(""),
            Line::from("  This focused prototype does not model a Service catalog."),
            Line::from("  Esc returns to Songdial's exact prior Destination."),
        ])
        .style(palette.base())
        .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_search(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    if let Some(row) = line_area(area, 0) {
        render_text(frame, row, "  SEARCH EVERYTHING", palette.title());
    }
    if let Some(row) = line_area(area, 1) {
        let cursor = if app.current.pane == ActivePane::Query {
            "_"
        } else {
            ""
        };
        let query = if app.current.query.is_empty() {
            format!("/ type night{cursor}")
        } else {
            format!("/ {}{cursor}", app.current.query)
        };
        render_text(frame, row, format!("  QUERY  {query}"), palette.amber());
    }
    let results = app.search_results();
    if let Some(row) = line_area(area, 2) {
        let status = if results.is_empty() {
            "Results update while typing; fixed demo query: night".into()
        } else {
            format!(
                "{} RESULTS | grouped by intent, Station, Playlist, and Track",
                results.len()
            )
        };
        render_text(frame, row, format!("  {status}"), palette.utility());
    }
    if results.is_empty() {
        if let Some(row) = line_area(area, 5) {
            render_text(
                frame,
                row,
                "  No demo matches yet. Type night to search both fictional Services.",
                palette.base(),
            );
        }
        return;
    }

    let mut row = 3u16;
    let mut previous_group = "";
    for (index, result) in results.iter().enumerate() {
        if result.group != previous_group {
            let Some(group_area) = line_area(area, row) else {
                break;
            };
            render_text(
                frame,
                group_area,
                format!("  -- {}", result.group),
                palette.utility(),
            );
            row += 1;
            previous_group = result.group;
        }
        let Some(result_area) = line_area(area, row) else {
            break;
        };
        let unavailable = match result.target {
            SearchTarget::Item(item) => !item_available(item),
            SearchTarget::Intent(_) => false,
        };
        render_named_row(
            frame,
            result_area,
            palette,
            app.current.pane == ActivePane::List && app.current.selected == index,
            result.title,
            result.subtitle,
            result.source,
            unavailable,
        );
        row += 1;
    }
}

fn render_queue(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    if let Some(row) = line_area(area, 0) {
        render_text(frame, row, "  NOW PLAYING & QUEUE", palette.title());
    }
    let Some(playback) = app.playback.as_ref() else {
        if let Some(row) = line_area(area, 2) {
            render_text(
                frame,
                row,
                "  [STOPPED] Nothing playing",
                palette.unavailable(),
            );
        }
        if let Some(row) = line_area(area, 4) {
            render_text(
                frame,
                row,
                "  Select a Track, Playlist, or Station and press p.",
                palette.base(),
            );
        }
        return;
    };

    if let Some(row) = line_area(area, 1) {
        render_text(
            frame,
            row,
            format!(
                "  CURRENT  {}  {}  [{}]",
                item_kind(playback.current),
                item_title(playback.current),
                item_source(playback.current)
            ),
            palette.green(),
        );
    }
    if let Some(row) = line_area(area, 2) {
        let state = match playback.state {
            PlayState::Playing => "PLAYING",
            PlayState::Paused => "PAUSED",
            PlayState::Live => "LIVE",
        };
        let progress = match playback.current {
            ItemRef::Track(id) => format!(
                "{} / {}",
                format_duration(playback.progress_seconds),
                format_duration(track(id).seconds)
            ),
            ItemRef::Station(_) => "continuous".into(),
            ItemRef::Playlist(_) => unreachable!(),
        };
        render_text(
            frame,
            row,
            format!("  [{state}]  {progress}  | Space toggles immediately"),
            if playback.state == PlayState::Paused {
                palette.amber()
            } else {
                palette.green()
            },
        );
    }
    if let Some(row) = line_area(area, 3) {
        render_text(
            frame,
            row,
            format!(
                "  SESSION  started from {} {}",
                item_kind(playback.origin),
                item_title(playback.origin)
            ),
            palette.utility(),
        );
    }
    if let Some(row) = line_area(area, 4) {
        let first = if playback.queue.is_empty() {
            0
        } else {
            app.current.scroll + 1
        };
        render_text(
            frame,
            row,
            format!(
                "  QUEUE  {} TRACKS | showing from {} | d removes selected",
                playback.queue.len(),
                first
            ),
            palette.utility(),
        );
    }
    for (index, track_id) in playback
        .queue
        .iter()
        .copied()
        .enumerate()
        .skip(app.current.scroll)
    {
        let Some(row_area) = line_area(area, 5 + (index - app.current.scroll) as u16) else {
            break;
        };
        render_item_row(
            frame,
            row_area,
            app,
            palette,
            ItemRef::Track(track_id),
            app.current.selected == index,
        );
    }
}

fn detail_item_lines(item: ItemRef) -> Vec<String> {
    match item {
        ItemRef::Track(id) => {
            let track = track(id);
            let mut lines = vec![
                "TRACK".into(),
                track.title.into(),
                track.artist.into(),
                format!("Source       [{}]", track.source),
                format!("Duration     {}", format_duration(track.seconds)),
                format!(
                    "Availability {}",
                    if track.available {
                        "AVAILABLE"
                    } else {
                        "UNAVAILABLE"
                    }
                ),
            ];
            if let Some(reason) = track.unavailable_reason {
                lines.push(String::new());
                lines.push(format!("Reason: {reason}"));
            }
            lines
        }
        ItemRef::Playlist(id) => {
            let playlist = playlist(id);
            vec![
                "PLAYLIST".into(),
                playlist.title.into(),
                format!("Source  [{}]", playlist.source),
                format!("Tracks  {}", playlist.track_ids.len()),
                String::new(),
                playlist.description.into(),
                String::new(),
                "Enter opens without playing.".into(),
                "p starts its first available Track and queues the remainder.".into(),
            ]
        }
        ItemRef::Station(id) => {
            let station = station(id);
            let mut lines = vec![
                "STATION".into(),
                station.title.into(),
                format!("Source  [{}]", station.source),
                format!("Style   {}", station.style),
                format!(
                    "State   {}",
                    if station.available {
                        "AVAILABLE"
                    } else {
                        "UNAVAILABLE"
                    }
                ),
                String::new(),
                station.description.into(),
            ];
            if let Some(reason) = station.unavailable_reason {
                lines.push(String::new());
                lines.push(format!("Reason: {reason}"));
            }
            lines
        }
    }
}

fn render_detail_lines(
    frame: &mut Frame<'_>,
    area: Rect,
    palette: Palette,
    mut lines: Vec<String>,
) {
    if area.width < 5 {
        return;
    }
    let inner = Rect::new(
        area.x + 2,
        area.y,
        area.width.saturating_sub(4),
        area.height,
    );
    lines.insert(0, "DETAIL LENS / READ ONLY".into());
    for (row_index, line) in lines.into_iter().enumerate() {
        let Some(row) = line_area(inner, row_index as u16) else {
            break;
        };
        let style = match row_index {
            0 => palette.utility(),
            2 => palette.title(),
            _ => palette.base(),
        };
        render_text(frame, row, line, style);
    }
}

fn render_details(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    let lines = match &app.current.destination {
        Destination::Home => {
            let (title, description) = HOME_ITEMS[app.current.selected];
            vec![
                "HOME DESTINATION".into(),
                (*title).into(),
                String::new(),
                (*description).into(),
                String::new(),
                "Wide mode adds explanation here.".into(),
                "Enter still opens the same Destination.".into(),
            ]
        }
        Destination::Intents => {
            let (title, description) = INTENTS[app.current.selected];
            vec![
                "LISTENING INTENT".into(),
                (*title).into(),
                String::new(),
                (*description).into(),
                String::new(),
                if app.current.selected == 0 {
                    "20 fixed Stations and Playlists in the populated path.".into()
                } else {
                    "Intentionally empty in this focused prototype.".into()
                },
            ]
        }
        Destination::Intent(0) => app
            .selected_item()
            .map(detail_item_lines)
            .unwrap_or_else(|| vec!["NO SELECTION".into()]),
        Destination::Intent(index) => vec![
            "LISTENING INTENT".into(),
            INTENTS[*index].0.into(),
            String::new(),
            "Only Deep Work is populated for this experiment.".into(),
        ],
        Destination::Stations | Destination::Playlists => app
            .selected_item()
            .map(detail_item_lines)
            .unwrap_or_else(|| vec!["NO SELECTION".into()]),
        Destination::Services => {
            let (name, detail) = if app.current.selected == 0 {
                (
                    "Morrow",
                    "Contains the long Night Ledger and one Night Geometry Track.",
                )
            } else {
                (
                    "Harbor",
                    "Contains apparent duplicates with distinct Source identity.",
                )
            };
            vec![
                "FICTIONAL SERVICE".into(),
                name.into(),
                String::new(),
                detail.into(),
            ]
        }
        Destination::Playlist(_) | Destination::NowPlaying => app
            .selected_item()
            .map(detail_item_lines)
            .unwrap_or_else(|| {
                vec![
                    "QUEUE".into(),
                    "No queued Track selected".into(),
                    String::new(),
                    "The current item remains first in the left pane.".into(),
                ]
            }),
        Destination::Station(id) => detail_item_lines(ItemRef::Station(id)),
        Destination::Track(id) => detail_item_lines(ItemRef::Track(id)),
        Destination::Service(name) => vec![
            "FICTIONAL SERVICE".into(),
            (*name).into(),
            String::new(),
            "Service browsing remains under Songdial's navigation stack.".into(),
        ],
        Destination::Search => {
            if app.current.pane == ActivePane::Query {
                vec![
                    "SEARCH QUERY".into(),
                    if app.current.query.is_empty() {
                        "Type night".into()
                    } else {
                        app.current.query.clone()
                    },
                    String::new(),
                    "Results update with each keystroke.".into(),
                    "Moving selection never changes playback.".into(),
                ]
            } else if let Some(result) = app.search_results().get(app.current.selected) {
                let mut lines = vec![
                    result.group.into(),
                    result.title.into(),
                    format!("Type    {}", result.subtitle),
                    format!("Source  [{}]", result.source),
                    String::new(),
                    "Enter opens without playing.".into(),
                ];
                if result.title.starts_with("Night Geometry") || result.title == "Night Ledger" {
                    lines.push(String::new());
                    lines.push(
                        "An apparent duplicate exists in the other Service; Source is identity."
                            .into(),
                    );
                }
                lines
            } else {
                vec!["SEARCH".into(), "No fixed results yet".into()]
            }
        }
    };
    render_detail_lines(frame, area, palette, lines);
}

fn render_now_playing(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    if area.height < 2 {
        return;
    }
    let top = line_area(area, 0).unwrap();
    let bottom = line_area(area, 1).unwrap();
    frame.render_widget(Block::default().style(palette.location()), top);
    frame.render_widget(Block::default().style(palette.base()), bottom);

    let Some(playback) = app.playback.as_ref() else {
        render_text(
            frame,
            top,
            " NOW PLAYING  [STOPPED]  Nothing playing",
            palette.location(),
        );
        render_text(
            frame,
            bottom,
            " No Playback session | choose a Playable item and press p",
            palette.utility(),
        );
        return;
    };

    let top_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(23),
            Constraint::Min(1),
            Constraint::Length(SOURCE_WIDTH),
        ])
        .split(top);
    let state = match playback.state {
        PlayState::Playing => "[PLAYING]",
        PlayState::Paused => "[PAUSED]",
        PlayState::Live => "[LIVE]",
    };
    let state_style = if playback.state == PlayState::Paused {
        palette.amber()
    } else {
        palette.green()
    };
    render_text(
        frame,
        top_columns[0],
        format!(" NOW PLAYING {state}"),
        state_style,
    );
    let current_label = match playback.current {
        ItemRef::Track(id) => {
            let track = track(id);
            format!("{} - {}", track.title, track.artist)
        }
        ItemRef::Station(id) => station(id).title.into(),
        ItemRef::Playlist(_) => unreachable!(),
    };
    render_text(frame, top_columns[1], current_label, palette.location());
    frame.render_widget(
        Paragraph::new(format!("[{}]", item_source(playback.current)))
            .style(palette.badge())
            .alignment(Alignment::Right),
        top_columns[2],
    );

    let progress = match playback.current {
        ItemRef::Track(id) => format!(
            "{} {}/{}",
            item_kind(playback.current),
            format_duration(playback.progress_seconds),
            format_duration(track(id).seconds)
        ),
        ItemRef::Station(_) => "STATION continuous".into(),
        ItemRef::Playlist(_) => unreachable!(),
    };
    render_text(
        frame,
        bottom,
        format!(
            " {progress} | Queue {} | from {} {}",
            playback.queue.len(),
            item_kind(playback.origin),
            item_title(playback.origin)
        ),
        palette.utility(),
    );
}

fn guide_actions(app: &App) -> &'static str {
    if app.show_help {
        return "Esc close help";
    }
    match app.current.destination {
        Destination::Home => "j/k move | Enter open | / search | n now playing | q quit",
        Destination::Search if app.current.pane == ActivePane::Query => {
            "type query | j/down results | Enter results | Esc back | ? all keys"
        }
        Destination::Search => "j/k move | Enter open | / edit query | Esc back | ? all keys",
        Destination::NowPlaying => "j/k queue | Enter open | p play | d remove | Esc back",
        Destination::Station(_) | Destination::Track(_) => {
            "p play | a append | Space pause | Esc back | ? all keys"
        }
        Destination::Playlist(_) => "j/k move | Enter inspect | p play list | a append | Esc back",
        _ => "j/k move | Enter open | p play | Esc back | ? all keys",
    }
}

fn render_guide(frame: &mut Frame<'_>, area: Rect, app: &App, palette: Palette) {
    if let Some(row) = line_area(area, 0) {
        render_text(
            frame,
            row,
            format!(" KEYS  {}", guide_actions(app)),
            palette.amber(),
        );
    }
    if let Some(row) = line_area(area, 1) {
        let count = app.selection_count();
        let item = if count == 0 {
            "-".into()
        } else {
            format!("{}/{}", app.current.selected + 1, count)
        };
        render_text(
            frame,
            row,
            format!(
                " STATE  DEST {} | ITEM {} | OFFSET {} | PANE {} | {}",
                app.current.destination.label(),
                item,
                app.current.scroll,
                app.current.pane.label(),
                app.notice
            ),
            palette.utility(),
        );
    }
}

fn render_help(frame: &mut Frame<'_>, area: Rect, palette: Palette) {
    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().style(palette.base()), area);
    let lines = vec![
        Line::from(Span::styled("  ALL KEYS", palette.title())),
        Line::from(""),
        Line::from("  j / k or arrows   Move selection"),
        Line::from("  Enter             Open without playing"),
        Line::from("  p                 Start a new Playback session"),
        Line::from("  a                 Append Track or Playlist Tracks"),
        Line::from("  Space             Toggle playing and paused"),
        Line::from("  /                 Open or focus Search"),
        Line::from("  n                 Open Now Playing and Queue"),
        Line::from("  d                 Remove selected Queue entry"),
        Line::from("  Esc               Close help or restore prior Destination"),
        Line::from("  q                 Quit from Home; always quits minimum guard"),
        Line::from("  ?                 Toggle this full key set"),
        Line::from(""),
        Line::from(Span::styled(
            "  Selection never changes playback. Opening never starts playback.",
            palette.amber(),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .style(palette.base())
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn press(app: &mut App, code: KeyCode) {
    let _ = app.handle_key(key(code));
}

fn press_down(app: &mut App, count: usize) {
    for _ in 0..count {
        press(app, KeyCode::Down);
    }
}

fn type_query(app: &mut App, query: &str) {
    for character in query.chars() {
        press(app, KeyCode::Char(character));
    }
}

fn scenario_intent_playing(no_color: bool) -> App {
    let mut app = App::new(no_color);
    app.set_size(80, 24);

    // Walkthrough 1: Home -> Listening intent -> Playlist details -> p.
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    press_down(&mut app, 8);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('p'));
    press(&mut app, KeyCode::Esc);

    // Browse elsewhere while Night Ledger keeps playing. This scrolls the
    // compact list while retaining the playing Playlist inside the viewport.
    press_down(&mut app, 8);

    // Walkthrough 2: opening and returning restores this exact navigation
    // state, even after the child Destination moves its own selection.
    let prior = app.current.clone();
    press(&mut app, KeyCode::Enter);
    press_down(&mut app, 2);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.current, prior, "Esc must restore exact NavState");
    app
}

fn scenario_search() -> App {
    let mut app = scenario_intent_playing(false);
    let playback_before = app.playback.clone();
    press(&mut app, KeyCode::Char('/'));
    type_query(&mut app, "night");
    press(&mut app, KeyCode::Down);
    press_down(&mut app, 6);

    // Walkthrough 3: Enter opens the Source-scoped duplicate but does not play.
    let search_state = app.current.clone();
    press(&mut app, KeyCode::Enter);
    assert!(matches!(
        app.current.destination,
        Destination::Track("night-geometry-harbor")
    ));
    assert_eq!(app.playback, playback_before, "opening changed playback");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.current, search_state, "Search state was not restored");
    app
}

fn scenario_queue() -> App {
    let mut app = scenario_intent_playing(false);
    press(&mut app, KeyCode::Char('n'));
    press_down(&mut app, 17);
    let before = app
        .playback
        .as_ref()
        .map(|playback| playback.queue.len())
        .unwrap_or(0);
    press(&mut app, KeyCode::Char('d'));
    let after = app
        .playback
        .as_ref()
        .map(|playback| playback.queue.len())
        .unwrap_or(0);
    assert_eq!(before - 1, after, "d must remove one Queue entry");
    assert!(app.current.scroll > 0, "Queue scenario must be scrolled");
    app
}

fn render_buffer(app: &App, width: u16, height: u16) -> Result<Buffer, Box<dyn Error>> {
    let mut rendered_app = app.clone();
    rendered_app.set_size(width, height);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render(frame, &rendered_app))?;
    Ok(terminal.backend().buffer().clone())
}

fn color_rgb(color: Color, fallback: [u8; 3]) -> [u8; 3] {
    match color {
        Color::Reset => fallback,
        Color::Black => [0, 0, 0],
        Color::Red => [205, 49, 49],
        Color::Green => [13, 188, 121],
        Color::Yellow => [229, 229, 16],
        Color::Blue => [36, 114, 200],
        Color::Magenta => [188, 63, 188],
        Color::Cyan => [17, 168, 205],
        Color::Gray => [204, 204, 204],
        Color::DarkGray => [102, 102, 102],
        Color::LightRed => [241, 76, 76],
        Color::LightGreen => [35, 209, 139],
        Color::LightYellow => [245, 245, 67],
        Color::LightBlue => [59, 142, 234],
        Color::LightMagenta => [214, 112, 214],
        Color::LightCyan => [41, 184, 219],
        Color::White => [242, 242, 242],
        Color::Indexed(index) => {
            let level = if index < 16 { 160 } else { index };
            [level, level, level]
        }
        Color::Rgb(red, green, blue) => [red, green, blue],
    }
}

fn blend(foreground: [u8; 3], background: [u8; 3], amount: f32) -> [u8; 3] {
    [
        (foreground[0] as f32 * amount + background[0] as f32 * (1.0 - amount)) as u8,
        (foreground[1] as f32 * amount + background[1] as f32 * (1.0 - amount)) as u8,
        (foreground[2] as f32 * amount + background[2] as f32 * (1.0 - amount)) as u8,
    ]
}

fn glyph(character: char) -> Option<[u8; 8]> {
    BASIC_FONTS
        .get(character)
        .or_else(|| LATIN_FONTS.get(character))
        .or_else(|| BOX_FONTS.get(character))
        .or_else(|| MISC_FONTS.get(character))
}

fn save_buffer_png(
    buffer: &Buffer,
    width: u16,
    height: u16,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    const CELL_WIDTH: u32 = 9;
    const CELL_HEIGHT: u32 = 16;
    let default_foreground = [230, 221, 201];
    let default_background = [16, 19, 21];
    let mut image = RgbImage::new(width as u32 * CELL_WIDTH, height as u32 * CELL_HEIGHT);

    for y in 0..height {
        for x in 0..width {
            let cell = &buffer[(x, y)];
            let mut foreground = color_rgb(cell.fg, default_foreground);
            let mut background = color_rgb(cell.bg, default_background);
            if cell.modifier.contains(Modifier::REVERSED) {
                std::mem::swap(&mut foreground, &mut background);
            }
            if cell.modifier.contains(Modifier::DIM) {
                foreground = blend(foreground, background, 0.58);
            }
            for pixel_y in 0..CELL_HEIGHT {
                for pixel_x in 0..CELL_WIDTH {
                    image.put_pixel(
                        x as u32 * CELL_WIDTH + pixel_x,
                        y as u32 * CELL_HEIGHT + pixel_y,
                        Rgb(background),
                    );
                }
            }

            let character = cell.symbol().chars().next().unwrap_or(' ');
            if let Some(bitmap) = glyph(character) {
                for (glyph_y, row) in bitmap.iter().enumerate() {
                    for glyph_x in 0..8u32 {
                        if row & (1 << glyph_x) != 0 {
                            for scale_y in 0..2u32 {
                                let pixel_x = x as u32 * CELL_WIDTH + glyph_x;
                                let pixel_y = y as u32 * CELL_HEIGHT + glyph_y as u32 * 2 + scale_y;
                                image.put_pixel(pixel_x, pixel_y, Rgb(foreground));
                                if cell.modifier.contains(Modifier::BOLD)
                                    && pixel_x + 1 < (x as u32 + 1) * CELL_WIDTH
                                {
                                    image.put_pixel(pixel_x + 1, pixel_y, Rgb(foreground));
                                }
                            }
                        }
                    }
                }
            }
            if cell.modifier.contains(Modifier::UNDERLINED) {
                let underline_y = (y as u32 + 1) * CELL_HEIGHT - 2;
                for pixel_x in 0..8u32 {
                    image.put_pixel(
                        x as u32 * CELL_WIDTH + pixel_x,
                        underline_y,
                        Rgb(foreground),
                    );
                }
            }
        }
    }
    image.save(path)?;
    Ok(())
}

fn capture(
    app: &App,
    width: u16,
    height: u16,
    directory: &Path,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let buffer = render_buffer(app, width, height)?;
    let path = directory.join(filename);
    save_buffer_png(&buffer, width, height, &path)?;
    println!("captured {filename} at {width}x{height}");
    Ok(())
}

fn capture_matrix() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("screenshots");
    fs::create_dir_all(&directory)?;

    let home = App::new(false);
    let intent = scenario_intent_playing(false);
    let intent_no_color = scenario_intent_playing(true);
    let search = scenario_search();
    let queue = scenario_queue();

    capture(&home, 80, 24, &directory, "80x24-home.png")?;
    capture(&intent, 80, 24, &directory, "80x24-intent-playing.png")?;
    capture(&search, 80, 24, &directory, "80x24-search.png")?;
    capture(&queue, 80, 24, &directory, "80x24-queue.png")?;
    capture(
        &intent_no_color,
        80,
        24,
        &directory,
        "80x24-intent-no-color.png",
    )?;
    capture(&home, 120, 40, &directory, "120x40-home.png")?;
    capture(&intent, 120, 40, &directory, "120x40-intent-playing.png")?;
    capture(&search, 120, 40, &directory, "120x40-search.png")?;
    capture(&queue, 120, 40, &directory, "120x40-queue.png")?;
    capture(&queue, 79, 23, &directory, "79x23-too-small.png")?;
    Ok(())
}

fn buffer_lines(buffer: &Buffer, width: u16, height: u16) -> Vec<String> {
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol().chars().next().unwrap_or(' '))
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

fn check(condition: bool, message: &str) -> Result<(), Box<dyn Error>> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn verify_walkthroughs() -> Result<(), Box<dyn Error>> {
    let home = App::new(false);
    let intent = scenario_intent_playing(false);
    let search = scenario_search();
    let queue = scenario_queue();

    let playback = intent
        .playback
        .as_ref()
        .expect("walkthrough starts playback");
    check(
        matches!(playback.origin, ItemRef::Playlist("night-ledger"))
            && matches!(playback.current, ItemRef::Track("still-room"))
            && playback.queue.len() == 19,
        "walkthrough 1 did not establish the Playlist Playback session",
    )?;
    check(
        intent.current.selected == 16 && intent.current.scroll > 0,
        "walkthrough 1 did not preserve playback while browsing elsewhere",
    )?;
    println!("PASS walkthrough 1: Playlist started; browsing selection moved independently.");
    println!("PASS walkthrough 2: Esc restored exact selection, offset, pane, and query state.");

    check(
        matches!(search.current.destination, Destination::Search)
            && search.current.query == "night"
            && search.current.selected == 6
            && search.current.pane == ActivePane::List
            && search.playback == intent.playback,
        "walkthrough 3 Search/open state is wrong",
    )?;
    println!("PASS walkthrough 3: grouped Search opened a result without changing playback.");

    let queue_count = queue.playback.as_ref().unwrap().queue.len();
    check(
        queue_count == 18 && queue.current.scroll > 0,
        "walkthrough 4 did not scroll and remove a Queue entry",
    )?;
    let mut returned = queue.clone();
    let expected_return = returned.back_stack.last().cloned().unwrap();
    press(&mut returned, KeyCode::Esc);
    check(
        returned.current == expected_return,
        "walkthrough 4 did not return to the exact prior state",
    )?;
    println!("PASS walkthrough 4: Queue scrolled, one entry removed, and Esc returned.");

    let before_resize = queue.state_fingerprint();
    for (width, height) in [(80, 24), (120, 40), (79, 23), (80, 24)] {
        let _ = render_buffer(&queue, width, height)?;
        check(
            queue.state_fingerprint() == before_resize,
            "rendering a resized frame mutated application state",
        )?;
    }
    println!("PASS walkthrough 5: 80x24 -> 120x40 -> 79x23 -> 80x24 preserved state.");

    let compact_home = buffer_lines(&render_buffer(&home, 80, 24)?, 80, 24);
    check(compact_home[0].contains("SONGDIAL"), "Location row missing")?;
    check(
        compact_home[20].contains("NOW PLAYING") && compact_home[22].contains("KEYS"),
        "persistent compact regions are not in their fixed rows",
    )?;

    let compact_intent_buffer = render_buffer(&intent, 80, 24)?;
    let compact_intent = buffer_lines(&compact_intent_buffer, 80, 24).join("\n");
    check(
        compact_intent.contains("SELECTED")
            && compact_intent.contains("PLAYING")
            && compact_intent.contains("[MORROW]")
            && compact_intent.contains("[HARBOR]"),
        "compact Intent lost selection, playback, or Source disambiguation",
    )?;

    let no_color_app = scenario_intent_playing(true);
    let no_color_buffer = render_buffer(&no_color_app, 80, 24)?;
    let no_color_text = buffer_lines(&no_color_buffer, 80, 24).join("\n");
    check(
        no_color_text.contains("SELECTED") && no_color_text.contains("PLAYING"),
        "NO_COLOR lost textual state labels",
    )?;
    check(
        no_color_buffer.content.iter().all(|cell| {
            !matches!(cell.fg, Color::Rgb(_, _, _)) && !matches!(cell.bg, Color::Rgb(_, _, _))
        }),
        "NO_COLOR emitted explicit RGB styles",
    )?;

    let compact_search = buffer_lines(&render_buffer(&search, 80, 24)?, 80, 24).join("\n");
    check(
        compact_search.contains("LISTENING INTENTS")
            && compact_search.contains("STATIONS")
            && compact_search.contains("PLAYLISTS")
            && compact_search.contains("TRACKS")
            && compact_search.contains("[MORROW]")
            && compact_search.contains("[HARBOR]"),
        "Search groups or protected Source badges are missing",
    )?;

    let wide_intent = buffer_lines(&render_buffer(&intent, 120, 40)?, 120, 40).join("\n");
    check(
        wide_intent.contains("DETAIL LENS / READ ONLY")
            && wide_intent.contains("SELECTED")
            && wide_intent.contains("PLAYING"),
        "wide mode did not add detail while preserving list state",
    )?;

    let compact_queue = buffer_lines(&render_buffer(&queue, 80, 24)?, 80, 24).join("\n");
    check(
        compact_queue.contains("CURRENT")
            && compact_queue.contains("QUEUE  18 TRACKS")
            && compact_queue.contains("OFFSET 4"),
        "Queue compact state is not usable or visibly scrolled",
    )?;

    let guard = buffer_lines(&render_buffer(&queue, 79, 23)?, 79, 23).join("\n");
    check(
        guard.contains("Current: 79x23 cells")
            && guard.contains("Required: 80x24 cells")
            && guard.contains("Resize to recover")
            && guard.contains("q quit"),
        "minimum-size guard lacks dimensions, recovery, or quit guidance",
    )?;

    println!("PASS gate: all seven pass/fail criteria have deterministic evidence.");
    Ok(())
}

fn run_tui(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    let initial = terminal.size()?;
    app.set_size(initial.width, initial.height);

    let result = (|| -> io::Result<()> {
        loop {
            terminal.draw(|frame| render(frame, &app))?;
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) if app.handle_key(key) => break,
                    Event::Resize(width, height) => app.set_size(width, height),
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;
    terminal.show_cursor()?;
    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let mode = env::args().nth(1);
    match mode.as_deref() {
        Some("--verify-walkthroughs") => verify_walkthroughs(),
        Some("--capture-matrix") => capture_matrix(),
        Some(other) => Err(format!(
            "unknown prototype option {other}; use --verify-walkthroughs or --capture-matrix"
        )
        .into()),
        None => run_tui(App::new(env::var_os("NO_COLOR").is_some())).map_err(Into::into),
    }
}
