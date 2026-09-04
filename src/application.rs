use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    Availability, CatalogId, DemoCatalog,
    catalog::{IntentMatch, Playlist, SearchResult, ServiceCatalogItem, Station, Track},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HomeChoice {
    ListeningIntents,
    Stations,
    Playlists,
    Services,
    Search,
}

impl HomeChoice {
    const ALL: [Self; 5] = [
        Self::ListeningIntents,
        Self::Stations,
        Self::Playlists,
        Self::Services,
        Self::Search,
    ];

    const fn text(self) -> HomeChoiceText {
        match self {
            Self::ListeningIntents => HomeChoiceText {
                label: "Mood & activity",
                detail: "Start from a Listening intent before choosing a Source.",
            },
            Self::Stations => HomeChoiceText {
                label: "Radio stations",
                detail: "Browse continuous Stations from every Source.",
            },
            Self::Playlists => HomeChoiceText {
                label: "My playlists",
                detail: "Browse personal and saved Playlists in one place.",
            },
            Self::Services => HomeChoiceText {
                label: "Browse services",
                detail: "Deliberately enter one fictional Service catalog.",
            },
            Self::Search => HomeChoiceText {
                label: "Search everything",
                detail: "Search every Source while keeping results grouped.",
            },
        }
    }
}

struct HomeChoiceText {
    label: &'static str,
    detail: &'static str,
}

struct DetailLensContent {
    kind: &'static str,
    title: String,
    facts: Vec<String>,
}

struct DenseCatalogRow<'a> {
    row: u16,
    selected: bool,
    playing: bool,
    kind: &'static str,
    title: &'a str,
    source_id: &'a CatalogId,
    detail: &'a str,
    availability: &'a Availability,
    status: &'a str,
}

type QueueAddition = Result<(Vec<CatalogId>, String), String>;

struct ServiceCatalogItemProjection<'a> {
    details_destination: Destination,
    playback_start: PlaybackStartOutcome,
    queue_addition: QueueAddition,
    queue_is_applicable: bool,
    playing: bool,
    kind: &'static str,
    title: &'a str,
    source_id: &'a CatalogId,
    detail: String,
    availability: &'a Availability,
    status: String,
}

impl ServiceCatalogItemProjection<'_> {
    fn can_play(&self) -> bool {
        matches!(self.playback_start, PlaybackStartOutcome::Ready(_))
    }

    fn can_queue(&self) -> bool {
        self.queue_addition.is_ok()
    }
}

struct SearchResultProjection<'a> {
    details_destination: Destination,
    playback_start: PlaybackStartOutcome,
    queue_addition: Option<QueueAddition>,
    queue_is_applicable: bool,
    playing: bool,
    kind: &'static str,
    title: &'a str,
    source_id: Option<&'a CatalogId>,
    availability: &'a Availability,
}

impl SearchResultProjection<'_> {
    fn can_play(&self) -> bool {
        matches!(self.playback_start, PlaybackStartOutcome::Ready(_))
    }

    fn can_queue(&self) -> bool {
        self.queue_addition.as_ref().is_some_and(Result::is_ok)
    }

    fn play_is_applicable(&self) -> bool {
        !matches!(
            self.playback_start,
            PlaybackStartOutcome::NoPlayableSelected
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    width: u16,
    height: u16,
}

impl Viewport {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationOptions {
    color_enabled: bool,
    motion_enabled: bool,
}

impl ApplicationOptions {
    #[must_use]
    pub const fn new(color_enabled: bool, motion_enabled: bool) -> Self {
        Self {
            color_enabled,
            motion_enabled,
        }
    }
}

impl Default for ApplicationOptions {
    fn default() -> Self {
        Self::new(true, true)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Up,
    Down,
    Enter,
    Backspace,
    Escape,
    CtrlC,
    Char(char),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Key(Key),
    Tick,
    Resize(Viewport),
    PlaybackLoaded(PlaybackRequestId),
    PlaybackFailed {
        request_id: PlaybackRequestId,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect {
    None,
    Quit,
    LoadPlayback(PlaybackRequest),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlaybackRequestId(u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaybackTarget {
    Station(CatalogId),
    Track(CatalogId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackRequest {
    id: PlaybackRequestId,
    target: PlaybackTarget,
}

impl PlaybackRequest {
    #[must_use]
    pub const fn id(&self) -> PlaybackRequestId {
        self.id
    }

    #[must_use]
    pub const fn target(&self) -> &PlaybackTarget {
        &self.target
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Destination {
    Home,
    ListeningIntents,
    ListeningIntent(CatalogId),
    Stations,
    Playlists,
    Services,
    ServiceCatalog(CatalogId),
    Search,
    StationDetails {
        origin: StationDetailsOrigin,
        station_id: CatalogId,
    },
    PlaylistDetails {
        origin: PlaylistDetailsOrigin,
        playlist_id: CatalogId,
    },
    NowPlaying,
    TrackDetails {
        origin: TrackDetailsOrigin,
        track_id: CatalogId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum StationDetailsOrigin {
    ListeningIntent(CatalogId),
    RadioStations,
    ServiceCatalog(CatalogId),
    Search,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PlaylistDetailsOrigin {
    ListeningIntent(CatalogId),
    MyPlaylists,
    ServiceCatalog(CatalogId),
    Search,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TrackDetailsOrigin {
    ServiceCatalog(CatalogId),
    Search,
    Queue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActivePane {
    List,
    Details,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DestinationSnapshot {
    destination: Destination,
    selection: usize,
    scroll_offset: usize,
    active_pane: ActivePane,
    query: String,
    query_focused: bool,
}

impl DestinationSnapshot {
    const fn new(destination: Destination) -> Self {
        let query_focused = matches!(destination, Destination::Search);
        let active_pane = match &destination {
            Destination::StationDetails { .. } | Destination::TrackDetails { .. } => {
                ActivePane::Details
            }
            Destination::Home
            | Destination::ListeningIntents
            | Destination::ListeningIntent(_)
            | Destination::Stations
            | Destination::Playlists
            | Destination::Services
            | Destination::ServiceCatalog(_)
            | Destination::Search
            | Destination::PlaylistDetails { .. }
            | Destination::NowPlaying => ActivePane::List,
        };
        Self {
            destination,
            selection: 0,
            scroll_offset: 0,
            active_pane,
            query: String::new(),
            query_focused,
        }
    }
}

pub struct Application {
    viewport: Viewport,
    options: ApplicationOptions,
    catalog: DemoCatalog,
    current: DestinationSnapshot,
    history: Vec<DestinationSnapshot>,
    help_visible: bool,
    playback: Option<PlaybackSession>,
    pending_playback: Option<PendingPlayback>,
    playback_feedback: Option<PlaybackFeedback>,
    next_playback_request_id: u64,
    motion_frame: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PlaybackFeedback {
    Brief(String),
    Persistent(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PlaybackSession {
    Station {
        station_id: CatalogId,
        state: StationPlaybackState,
    },
    Track {
        current_track_id: CatalogId,
        origin_playlist_id: Option<CatalogId>,
        queue: Queue,
        elapsed_seconds: u16,
        state: TrackPlaybackState,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Queue {
    tracks: VecDeque<CatalogId>,
}

impl Queue {
    fn from_tracks(tracks: impl IntoIterator<Item = CatalogId>) -> Self {
        Self {
            tracks: tracks.into_iter().collect(),
        }
    }

    fn len(&self) -> usize {
        self.tracks.len()
    }

    fn get(&self, index: usize) -> Option<&CatalogId> {
        self.tracks.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = &CatalogId> {
        self.tracks.iter()
    }

    fn append(&mut self, tracks: impl IntoIterator<Item = CatalogId>) {
        self.tracks.extend(tracks);
    }

    fn remove(&mut self, index: usize) -> Option<CatalogId> {
        self.tracks.remove(index)
    }

    fn pop_next(&mut self) -> Option<CatalogId> {
        self.tracks.pop_front()
    }

    fn tracks_after(&self, index: usize) -> Self {
        Self::from_tracks(self.tracks.iter().skip(index + 1).cloned())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StationPlaybackState {
    Live,
    Paused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrackPlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingPlayback {
    request: PlaybackRequest,
    candidate: PlaybackSession,
}

enum PlaybackStartOutcome {
    NoPlayableSelected,
    Rejected(String),
    Ready(PlaybackSession),
}

impl Application {
    #[must_use]
    pub fn new(viewport: Viewport) -> Self {
        Self::with_catalog_and_options(
            viewport,
            DemoCatalog::fixed(),
            ApplicationOptions::default(),
        )
    }

    #[must_use]
    pub fn with_options(viewport: Viewport, options: ApplicationOptions) -> Self {
        Self::with_catalog_and_options(viewport, DemoCatalog::fixed(), options)
    }

    #[must_use]
    pub fn with_catalog(viewport: Viewport, catalog: DemoCatalog) -> Self {
        Self::with_catalog_and_options(viewport, catalog, ApplicationOptions::default())
    }

    #[must_use]
    pub fn with_catalog_and_options(
        viewport: Viewport,
        catalog: DemoCatalog,
        options: ApplicationOptions,
    ) -> Self {
        Self {
            viewport,
            options,
            catalog,
            current: DestinationSnapshot::new(Destination::Home),
            history: Vec::new(),
            help_visible: false,
            playback: None,
            pending_playback: None,
            playback_feedback: None,
            next_playback_request_id: 1,
            motion_frame: 0,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Effect {
        if let Event::Resize(viewport) = event {
            self.viewport = viewport;
            return Effect::None;
        }

        if !self.viewport.is_supported() {
            return match event {
                Event::Key(Key::Char('q') | Key::CtrlC) => Effect::Quit,
                Event::PlaybackLoaded(request_id) => {
                    self.finish_playback_load(request_id);
                    Effect::None
                }
                Event::PlaybackFailed { request_id, reason } => {
                    self.fail_playback_load(request_id, reason);
                    Effect::None
                }
                Event::Key(_) | Event::Tick => Effect::None,
                Event::Resize(_) => unreachable!("resize events handled above"),
            };
        }

        let Event::Key(key) = event else {
            return match event {
                Event::Tick => {
                    if matches!(
                        self.playback_feedback.as_ref(),
                        Some(PlaybackFeedback::Brief(_))
                    ) {
                        self.playback_feedback = None;
                    }
                    if self.options.motion_enabled && self.pending_playback.is_some() {
                        self.motion_frame = (self.motion_frame + 1) % 4;
                    }
                    self.advance_playback_tick();
                    Effect::None
                }
                Event::PlaybackLoaded(request_id) => {
                    self.finish_playback_load(request_id);
                    Effect::None
                }
                Event::PlaybackFailed { request_id, reason } => {
                    self.fail_playback_load(request_id, reason);
                    Effect::None
                }
                Event::Key(_) => unreachable!("key event handled above"),
                Event::Resize(_) => unreachable!("resize events handled above"),
            };
        };

        if key == Key::CtrlC {
            return Effect::Quit;
        }

        if self.help_visible {
            return match key {
                Key::Char('q') => Effect::Quit,
                Key::Escape | Key::Char('?') => {
                    self.help_visible = false;
                    Effect::None
                }
                _ => Effect::None,
            };
        }

        if self.current.destination == Destination::Search && self.current.query_focused {
            match key {
                Key::Escape => self.current.query_focused = false,
                Key::Backspace => {
                    self.current.query.pop();
                    self.current.selection = 0;
                    self.current.scroll_offset = 0;
                }
                Key::Char('/') => {}
                Key::Char('?') => self.help_visible = true,
                Key::Char(character) => {
                    self.current.query.push(character);
                    self.current.selection = 0;
                    self.current.scroll_offset = 0;
                }
                Key::Down => self.move_selection_down(),
                Key::Up => self.move_selection_up(),
                Key::Enter => self.open_selected(),
                Key::CtrlC => unreachable!("Ctrl+C handled above"),
            }
            return Effect::None;
        }

        if key == Key::Char('q') {
            return Effect::Quit;
        }

        if key == Key::Char('?') {
            self.help_visible = !self.help_visible;
            return Effect::None;
        }

        match key {
            Key::Down | Key::Char('j') => self.move_selection_down(),
            Key::Up | Key::Char('k') => self.move_selection_up(),
            Key::Enter => self.open_selected(),
            Key::Escape => self.restore_previous_destination(),
            Key::Char('p') => return self.start_selected_playback(),
            Key::Char('a') => self.add_selected_to_queue(),
            Key::Char('d') => self.remove_selected_from_queue(),
            Key::Char('n') => self.open_now_playing(),
            Key::Char('/') => self.open_or_focus_search(),
            Key::Char(' ') => self.toggle_playback(),
            Key::Char(_) | Key::Backspace | Key::CtrlC => {}
        }

        Effect::None
    }

    fn remove_selected_from_queue(&mut self) {
        if self.current.destination != Destination::NowPlaying {
            return;
        }
        let Some(PlaybackSession::Track { queue, .. }) = &mut self.playback else {
            return;
        };
        let Some(removed_track_id) = queue.remove(self.current.selection) else {
            return;
        };
        let queue_len = queue.len();
        self.current.selection = self.current.selection.min(queue_len.saturating_sub(1));
        if queue_len == 0 {
            self.current.scroll_offset = 0;
        } else {
            self.keep_selection_visible();
        }
        let removed_track = self
            .catalog
            .track(&removed_track_id)
            .expect("Queue should reference a catalog Track");
        let suffix = if queue_len == 1 { "" } else { "s" };
        self.playback_feedback = Some(PlaybackFeedback::Brief(format!(
            "Removed {} from Queue. Queue {} Track{suffix}",
            removed_track.name(),
            queue_len
        )));
    }

    fn add_selected_to_queue(&mut self) {
        let addition = match &self.current.destination {
            Destination::PlaylistDetails { playlist_id, .. } => {
                let Some(playlist) = self.catalog.playlist(playlist_id) else {
                    return;
                };
                if playlist.track_ids().is_empty() {
                    self.playlist_queue_addition(playlist)
                } else {
                    let Some(track) = playlist
                        .track_ids()
                        .get(self.current.selection)
                        .and_then(|track_id| self.catalog.track(track_id))
                    else {
                        return;
                    };
                    Self::track_queue_addition(track)
                }
            }
            Destination::ListeningIntent(intent_id) => {
                let intent_matches = self.catalog.intent_matches(intent_id);
                let Some(intent_match) = intent_matches.get(self.current.selection) else {
                    return;
                };
                match intent_match {
                    IntentMatch::Station(_) => {
                        Err("Stations are continuous and cannot be queued.".to_owned())
                    }
                    IntentMatch::Playlist(playlist) => self.playlist_queue_addition(playlist),
                }
            }
            Destination::Playlists => {
                let Some(playlist) = self.catalog.playlists().get(self.current.selection) else {
                    return;
                };
                self.playlist_queue_addition(playlist)
            }
            Destination::ServiceCatalog(service_id) => {
                let service_catalog = self.catalog.service_catalog(service_id);
                let Some(item) = service_catalog.items.get(self.current.selection) else {
                    return;
                };
                self.service_catalog_item_projection(service_id, *item)
                    .queue_addition
            }
            Destination::Search => {
                let results = self.catalog.search(&self.current.query);
                let Some(result) = results.items.get(self.current.selection).copied() else {
                    return;
                };
                let Some(addition) = self.search_result_projection(result).queue_addition else {
                    return;
                };
                addition
            }
            Destination::TrackDetails {
                origin: TrackDetailsOrigin::ServiceCatalog(_) | TrackDetailsOrigin::Search,
                track_id,
            } => {
                let Some(track) = self.catalog.track(track_id) else {
                    return;
                };
                Self::track_queue_addition(track)
            }
            Destination::Home
            | Destination::ListeningIntents
            | Destination::NowPlaying
            | Destination::TrackDetails {
                origin: TrackDetailsOrigin::Queue,
                ..
            } => return,
            Destination::Services => return,
            Destination::Stations | Destination::StationDetails { .. } => {
                Err("Stations are continuous and cannot be queued.".to_owned())
            }
        };
        let (track_ids, confirmation) = match addition {
            Ok(addition) => addition,
            Err(reason) => {
                self.playback_feedback = Some(PlaybackFeedback::Persistent(format!(
                    "Cannot add: {reason}"
                )));
                return;
            }
        };
        let queue = match &mut self.playback {
            Some(PlaybackSession::Track { queue, .. }) => queue,
            Some(PlaybackSession::Station { .. }) => {
                self.playback_feedback = Some(PlaybackFeedback::Persistent(
                    "Cannot add: Continuous Stations stay LIVE with an empty Queue.".to_owned(),
                ));
                return;
            }
            None => {
                self.playback_feedback = Some(PlaybackFeedback::Persistent(
                    "Cannot add: Start a Track or Playlist before building a Queue.".to_owned(),
                ));
                return;
            }
        };

        queue.append(track_ids);
        let suffix = if queue.len() == 1 { "" } else { "s" };
        self.playback_feedback = Some(PlaybackFeedback::Brief(format!(
            "{confirmation} Queue {} Track{suffix}",
            queue.len()
        )));
    }

    fn playlist_queue_addition(
        &self,
        playlist: &Playlist,
    ) -> Result<(Vec<CatalogId>, String), String> {
        let track_ids = playlist
            .track_ids()
            .iter()
            .filter(|track_id| {
                self.catalog
                    .track(track_id)
                    .is_some_and(|track| track.availability() == &Availability::Available)
            })
            .cloned()
            .collect::<Vec<_>>();
        let added_count = track_ids.len();
        if added_count == 0 {
            return Err(format!("{} has no playable Tracks.", playlist.name()));
        }
        let suffix = if added_count == 1 { "" } else { "s" };
        Ok((
            track_ids,
            format!(
                "Added {} Track{suffix} from {}.",
                added_count,
                playlist.name()
            ),
        ))
    }

    fn track_queue_addition(track: &Track) -> Result<(Vec<CatalogId>, String), String> {
        match track.availability() {
            Availability::Available => Ok((
                vec![track.id().clone()],
                format!("Added {} to Queue.", track.name()),
            )),
            Availability::Loading => Err(format!("{} is still loading.", track.name())),
            Availability::Unavailable(reason) => Err(reason.clone()),
        }
    }

    fn track_playback_is_active(&self, track_id: &CatalogId) -> bool {
        matches!(
            &self.playback,
            Some(PlaybackSession::Track {
                current_track_id,
                state,
                ..
            }) if current_track_id == track_id && *state != TrackPlaybackState::Stopped
        )
    }

    fn playlist_playback_is_active(&self, playlist_id: &CatalogId) -> bool {
        matches!(
            &self.playback,
            Some(PlaybackSession::Track {
                origin_playlist_id: Some(origin_playlist_id),
                state,
                ..
            }) if origin_playlist_id == playlist_id && *state != TrackPlaybackState::Stopped
        )
    }

    fn service_catalog_item_projection<'a>(
        &self,
        service_id: &CatalogId,
        item: ServiceCatalogItem<'a>,
    ) -> ServiceCatalogItemProjection<'a> {
        match item {
            ServiceCatalogItem::Station(station) => ServiceCatalogItemProjection {
                details_destination: Destination::StationDetails {
                    origin: StationDetailsOrigin::ServiceCatalog(service_id.clone()),
                    station_id: station.id().clone(),
                },
                playback_start: Self::station_playback_start(station),
                queue_addition: Err("Stations are continuous and cannot be queued.".to_owned()),
                queue_is_applicable: false,
                playing: matches!(
                    &self.playback,
                    Some(PlaybackSession::Station { station_id, .. })
                        if station_id == station.id()
                ),
                kind: "STATION",
                title: station.name(),
                source_id: station.source_id(),
                detail: station.style().to_owned(),
                availability: station.availability(),
                status: Self::availability_label(station.availability()).to_owned(),
            },
            ServiceCatalogItem::Playlist(playlist) => {
                let track_count = playlist.track_ids().len();
                let suffix = if track_count == 1 { "" } else { "s" };
                ServiceCatalogItemProjection {
                    details_destination: Destination::PlaylistDetails {
                        origin: PlaylistDetailsOrigin::ServiceCatalog(service_id.clone()),
                        playlist_id: playlist.id().clone(),
                    },
                    playback_start: self.playlist_playback_start(playlist),
                    queue_addition: self.playlist_queue_addition(playlist),
                    queue_is_applicable: true,
                    playing: self.playlist_playback_is_active(playlist.id()),
                    kind: "PLAYLIST",
                    title: playlist.name(),
                    source_id: playlist.source_id(),
                    detail: format!("{track_count} Track{suffix}"),
                    availability: playlist.availability(),
                    status: if track_count == 0 {
                        "EMPTY"
                    } else {
                        Self::availability_label(playlist.availability())
                    }
                    .to_owned(),
                }
            }
            ServiceCatalogItem::Track(track) => ServiceCatalogItemProjection {
                details_destination: Destination::TrackDetails {
                    origin: TrackDetailsOrigin::ServiceCatalog(service_id.clone()),
                    track_id: track.id().clone(),
                },
                playback_start: Self::track_playback_start(track),
                queue_addition: Self::track_queue_addition(track),
                queue_is_applicable: true,
                playing: self.track_playback_is_active(track.id()),
                kind: "TRACK",
                title: track.name(),
                source_id: track.source_id(),
                detail: track.creator().to_owned(),
                availability: track.availability(),
                status: Self::track_row_status(track),
            },
        }
    }

    fn search_result_projection<'a>(&self, result: SearchResult<'a>) -> SearchResultProjection<'a> {
        match result {
            SearchResult::ListeningIntent(intent) => SearchResultProjection {
                details_destination: Destination::ListeningIntent(intent.id().clone()),
                playback_start: PlaybackStartOutcome::NoPlayableSelected,
                queue_addition: None,
                queue_is_applicable: false,
                playing: false,
                kind: "INTENT",
                title: intent.name(),
                source_id: None,
                availability: &Availability::Available,
            },
            SearchResult::Station(station) => SearchResultProjection {
                details_destination: Destination::StationDetails {
                    origin: StationDetailsOrigin::Search,
                    station_id: station.id().clone(),
                },
                playback_start: Self::station_playback_start(station),
                queue_addition: Some(Err(
                    "Stations are continuous and cannot be queued.".to_owned()
                )),
                queue_is_applicable: false,
                playing: matches!(
                    &self.playback,
                    Some(PlaybackSession::Station { station_id, .. })
                        if station_id == station.id()
                ),
                kind: "STATION",
                title: station.name(),
                source_id: Some(station.source_id()),
                availability: station.availability(),
            },
            SearchResult::Playlist(playlist) => SearchResultProjection {
                details_destination: Destination::PlaylistDetails {
                    origin: PlaylistDetailsOrigin::Search,
                    playlist_id: playlist.id().clone(),
                },
                playback_start: self.playlist_playback_start(playlist),
                queue_addition: Some(self.playlist_queue_addition(playlist)),
                queue_is_applicable: true,
                playing: self.playlist_playback_is_active(playlist.id()),
                kind: "PLAYLIST",
                title: playlist.name(),
                source_id: Some(playlist.source_id()),
                availability: playlist.availability(),
            },
            SearchResult::Track(track) => SearchResultProjection {
                details_destination: Destination::TrackDetails {
                    origin: TrackDetailsOrigin::Search,
                    track_id: track.id().clone(),
                },
                playback_start: Self::track_playback_start(track),
                queue_addition: Some(Self::track_queue_addition(track)),
                queue_is_applicable: true,
                playing: self.track_playback_is_active(track.id()),
                kind: "TRACK",
                title: track.name(),
                source_id: Some(track.source_id()),
                availability: track.availability(),
            },
        }
    }

    fn advance_playback_tick(&mut self) {
        let Some(PlaybackSession::Track {
            current_track_id,
            queue,
            elapsed_seconds,
            state,
            ..
        }) = &mut self.playback
        else {
            return;
        };
        if *state != TrackPlaybackState::Playing {
            return;
        }
        let duration = self
            .catalog
            .track(current_track_id)
            .expect("playback session should reference a catalog Track")
            .duration_seconds();
        let next_elapsed = elapsed_seconds.saturating_add(1).min(duration);
        if next_elapsed < duration {
            *elapsed_seconds = next_elapsed;
            return;
        }

        while let Some(next_track_id) = queue.pop_next() {
            Self::shift_queue_snapshot_after_advance(&mut self.current);
            for snapshot in &mut self.history {
                Self::shift_queue_snapshot_after_advance(snapshot);
            }
            let next_track = self
                .catalog
                .track(&next_track_id)
                .expect("Queue should reference a catalog Track");
            if next_track.availability() != &Availability::Available {
                continue;
            }
            *current_track_id = next_track_id;
            *elapsed_seconds = 0;
            return;
        }

        *elapsed_seconds = duration;
        *state = TrackPlaybackState::Stopped;
    }

    fn shift_queue_snapshot_after_advance(snapshot: &mut DestinationSnapshot) {
        if snapshot.destination == Destination::NowPlaying {
            snapshot.selection = snapshot.selection.saturating_sub(1);
            snapshot.scroll_offset = snapshot.scroll_offset.saturating_sub(1);
        }
    }

    fn toggle_playback(&mut self) {
        match &mut self.playback {
            Some(PlaybackSession::Track {
                state,
                elapsed_seconds,
                ..
            }) => {
                *state = match state {
                    TrackPlaybackState::Playing => TrackPlaybackState::Paused,
                    TrackPlaybackState::Paused => TrackPlaybackState::Playing,
                    TrackPlaybackState::Stopped => {
                        *elapsed_seconds = 0;
                        TrackPlaybackState::Playing
                    }
                };
            }
            Some(PlaybackSession::Station { state, .. }) => {
                *state = match state {
                    StationPlaybackState::Live => StationPlaybackState::Paused,
                    StationPlaybackState::Paused => StationPlaybackState::Live,
                };
            }
            None => {}
        }
    }

    fn start_selected_playback(&mut self) -> Effect {
        let candidate = match self.selected_playback_start() {
            PlaybackStartOutcome::NoPlayableSelected => return Effect::None,
            PlaybackStartOutcome::Rejected(reason) => {
                self.pending_playback = None;
                self.playback_feedback = Some(PlaybackFeedback::Persistent(format!(
                    "Cannot play: {reason}"
                )));
                return Effect::None;
            }
            PlaybackStartOutcome::Ready(candidate) => candidate,
        };

        let target = match &candidate {
            PlaybackSession::Station { station_id, .. } => {
                PlaybackTarget::Station(station_id.clone())
            }
            PlaybackSession::Track {
                current_track_id, ..
            } => PlaybackTarget::Track(current_track_id.clone()),
        };
        let request = PlaybackRequest {
            id: PlaybackRequestId(self.next_playback_request_id),
            target,
        };
        self.next_playback_request_id += 1;
        self.playback_feedback = None;
        self.motion_frame = 0;
        self.pending_playback = Some(PendingPlayback {
            request: request.clone(),
            candidate,
        });
        Effect::LoadPlayback(request)
    }

    fn selected_playback_start(&self) -> PlaybackStartOutcome {
        match &self.current.destination {
            Destination::ListeningIntent(intent_id) => self
                .catalog
                .intent_matches(intent_id)
                .get(self.current.selection)
                .map_or(
                    PlaybackStartOutcome::NoPlayableSelected,
                    |intent_match| match intent_match {
                        IntentMatch::Station(station) => Self::station_playback_start(station),
                        IntentMatch::Playlist(playlist) => self.playlist_playback_start(playlist),
                    },
                ),
            Destination::Stations => self
                .catalog
                .stations()
                .get(self.current.selection)
                .map_or(PlaybackStartOutcome::NoPlayableSelected, |station| {
                    Self::station_playback_start(station)
                }),
            Destination::Playlists => self
                .catalog
                .playlists()
                .get(self.current.selection)
                .map_or(PlaybackStartOutcome::NoPlayableSelected, |playlist| {
                    self.playlist_playback_start(playlist)
                }),
            Destination::ServiceCatalog(service_id) => {
                let service_catalog = self.catalog.service_catalog(service_id);
                service_catalog.items.get(self.current.selection).map_or(
                    PlaybackStartOutcome::NoPlayableSelected,
                    |item| {
                        self.service_catalog_item_projection(service_id, *item)
                            .playback_start
                    },
                )
            }
            Destination::Search => self
                .catalog
                .search(&self.current.query)
                .items
                .get(self.current.selection)
                .copied()
                .map_or(PlaybackStartOutcome::NoPlayableSelected, |result| {
                    self.search_result_projection(result).playback_start
                }),
            Destination::StationDetails { station_id, .. } => self
                .catalog
                .station(station_id)
                .map_or(PlaybackStartOutcome::NoPlayableSelected, |station| {
                    Self::station_playback_start(station)
                }),
            Destination::PlaylistDetails { playlist_id, .. } => {
                let Some(playlist) = self.catalog.playlist(playlist_id) else {
                    return PlaybackStartOutcome::NoPlayableSelected;
                };
                if playlist.track_ids().is_empty() {
                    self.playlist_playback_start(playlist)
                } else {
                    playlist
                        .track_ids()
                        .get(self.current.selection)
                        .and_then(|track_id| self.catalog.track(track_id))
                        .map_or(PlaybackStartOutcome::NoPlayableSelected, |track| {
                            Self::track_playback_start(track)
                        })
                }
            }
            Destination::NowPlaying => {
                let Some(PlaybackSession::Track { queue, .. }) = &self.playback else {
                    return PlaybackStartOutcome::NoPlayableSelected;
                };
                let Some(track_id) = queue.get(self.current.selection) else {
                    return PlaybackStartOutcome::NoPlayableSelected;
                };
                let track = self
                    .catalog
                    .track(track_id)
                    .expect("Queue should reference a catalog Track");
                if let Availability::Unavailable(reason) = track.availability() {
                    return PlaybackStartOutcome::Rejected(reason.clone());
                }
                PlaybackStartOutcome::Ready(PlaybackSession::Track {
                    current_track_id: track_id.clone(),
                    origin_playlist_id: None,
                    queue: queue.tracks_after(self.current.selection),
                    elapsed_seconds: 0,
                    state: TrackPlaybackState::Playing,
                })
            }
            Destination::TrackDetails {
                origin: TrackDetailsOrigin::ServiceCatalog(_) | TrackDetailsOrigin::Search,
                track_id,
            } => self
                .catalog
                .track(track_id)
                .map_or(PlaybackStartOutcome::NoPlayableSelected, |track| {
                    Self::track_playback_start(track)
                }),
            Destination::Home
            | Destination::ListeningIntents
            | Destination::Services
            | Destination::TrackDetails {
                origin: TrackDetailsOrigin::Queue,
                ..
            } => PlaybackStartOutcome::NoPlayableSelected,
        }
    }

    fn station_playback_start(station: &Station) -> PlaybackStartOutcome {
        if let Availability::Unavailable(reason) = station.availability() {
            return PlaybackStartOutcome::Rejected(reason.clone());
        }
        PlaybackStartOutcome::Ready(PlaybackSession::Station {
            station_id: station.id().clone(),
            state: StationPlaybackState::Live,
        })
    }

    fn track_playback_start(track: &Track) -> PlaybackStartOutcome {
        if let Availability::Unavailable(reason) = track.availability() {
            return PlaybackStartOutcome::Rejected(reason.clone());
        }
        PlaybackStartOutcome::Ready(PlaybackSession::Track {
            current_track_id: track.id().clone(),
            origin_playlist_id: None,
            queue: Queue::default(),
            elapsed_seconds: 0,
            state: TrackPlaybackState::Playing,
        })
    }

    fn playlist_playback_start(&self, playlist: &Playlist) -> PlaybackStartOutcome {
        if playlist.track_ids().is_empty() {
            return PlaybackStartOutcome::Rejected(format!("{} has no Tracks.", playlist.name()));
        }
        let Some(first_available) = playlist.track_ids().iter().position(|track_id| {
            self.catalog
                .track(track_id)
                .is_some_and(|track| track.availability() == &Availability::Available)
        }) else {
            return PlaybackStartOutcome::Rejected(format!(
                "{} has no available Tracks.",
                playlist.name()
            ));
        };
        PlaybackStartOutcome::Ready(PlaybackSession::Track {
            current_track_id: playlist.track_ids()[first_available].clone(),
            origin_playlist_id: Some(playlist.id().clone()),
            queue: Queue::from_tracks(playlist.track_ids()[first_available + 1..].iter().cloned()),
            elapsed_seconds: 0,
            state: TrackPlaybackState::Playing,
        })
    }

    fn finish_playback_load(&mut self, request_id: PlaybackRequestId) {
        let Some(pending) = self
            .pending_playback
            .take_if(|pending| pending.request.id == request_id)
        else {
            return;
        };
        self.playback = Some(pending.candidate);
        self.playback_feedback = None;
        Self::reset_queue_snapshot_after_playback_replacement(&mut self.current);
        for snapshot in &mut self.history {
            Self::reset_queue_snapshot_after_playback_replacement(snapshot);
        }
    }

    fn reset_queue_snapshot_after_playback_replacement(snapshot: &mut DestinationSnapshot) {
        if snapshot.destination == Destination::NowPlaying {
            snapshot.selection = 0;
            snapshot.scroll_offset = 0;
        }
    }

    fn fail_playback_load(&mut self, request_id: PlaybackRequestId, reason: String) {
        if self
            .pending_playback
            .take_if(|pending| pending.request.id == request_id)
            .is_some()
        {
            self.playback_feedback = Some(PlaybackFeedback::Persistent(format!(
                "Playback failed: {reason}"
            )));
        }
    }

    fn move_selection_down(&mut self) {
        let item_count = self.current_item_count();
        if item_count == 0 {
            return;
        }

        self.current.selection = (self.current.selection + 1).min(item_count - 1);
        self.keep_selection_visible();
    }

    fn move_selection_up(&mut self) {
        self.current.selection = self.current.selection.saturating_sub(1);
        self.keep_selection_visible();
    }

    fn keep_selection_visible(&mut self) {
        let visible_items = match &self.current.destination {
            Destination::ListeningIntent(_) => self.dense_visible_items(),
            Destination::Stations => self.dense_visible_items(),
            Destination::Playlists => self.dense_visible_items(),
            Destination::ServiceCatalog(_) => self.dense_visible_items(),
            Destination::Search => self.search_visible_items(),
            Destination::PlaylistDetails { .. } => self.playlist_visible_items(),
            Destination::NowPlaying => self.queue_visible_items(),
            Destination::Home => HomeChoice::ALL.len(),
            Destination::ListeningIntents => self.catalog.listening_intents().len(),
            Destination::Services => self.catalog.services().len(),
            Destination::StationDetails { .. } | Destination::TrackDetails { .. } => 0,
        };

        if visible_items == 0 || self.current.selection < self.current.scroll_offset {
            self.current.scroll_offset = self.current.selection;
        } else if self.current.selection >= self.current.scroll_offset + visible_items {
            self.current.scroll_offset = self.current.selection + 1 - visible_items;
        }
    }

    fn current_item_count(&self) -> usize {
        match &self.current.destination {
            Destination::Home => HomeChoice::ALL.len(),
            Destination::ListeningIntents => self.catalog.listening_intents().len(),
            Destination::Stations => self.catalog.stations().len(),
            Destination::Playlists => self.catalog.playlists().len(),
            Destination::Services => self.catalog.services().len(),
            Destination::ServiceCatalog(service_id) => {
                self.catalog.service_catalog(service_id).items.len()
            }
            Destination::Search => self.catalog.search(&self.current.query).items.len(),
            Destination::ListeningIntent(intent_id) => self.catalog.intent_matches(intent_id).len(),
            Destination::PlaylistDetails { playlist_id, .. } => self
                .catalog
                .playlist(playlist_id)
                .map_or(0, |playlist| playlist.track_ids().len()),
            Destination::NowPlaying => match &self.playback {
                Some(PlaybackSession::Track { queue, .. }) => queue.len(),
                Some(PlaybackSession::Station { .. }) | None => 0,
            },
            Destination::StationDetails { .. } | Destination::TrackDetails { .. } => 0,
        }
    }

    fn open_selected(&mut self) {
        let destination = match self.current.destination.clone() {
            Destination::Home => {
                let choice = self.selected_home_choice();
                match choice {
                    HomeChoice::ListeningIntents => Destination::ListeningIntents,
                    HomeChoice::Stations => Destination::Stations,
                    HomeChoice::Playlists => Destination::Playlists,
                    HomeChoice::Services => Destination::Services,
                    HomeChoice::Search => Destination::Search,
                }
            }
            Destination::ListeningIntents => self
                .catalog
                .listening_intents()
                .get(self.current.selection)
                .map(|intent| Destination::ListeningIntent(intent.id().clone()))
                .unwrap_or(Destination::ListeningIntents),
            Destination::ListeningIntent(intent_id) => {
                match self
                    .catalog
                    .intent_matches(&intent_id)
                    .get(self.current.selection)
                {
                    Some(IntentMatch::Station(station)) => Destination::StationDetails {
                        origin: StationDetailsOrigin::ListeningIntent(intent_id),
                        station_id: station.id().clone(),
                    },
                    Some(IntentMatch::Playlist(playlist)) => Destination::PlaylistDetails {
                        origin: PlaylistDetailsOrigin::ListeningIntent(intent_id),
                        playlist_id: playlist.id().clone(),
                    },
                    None => return,
                }
            }
            Destination::Stations => {
                let Some(station) = self.catalog.stations().get(self.current.selection) else {
                    return;
                };
                Destination::StationDetails {
                    origin: StationDetailsOrigin::RadioStations,
                    station_id: station.id().clone(),
                }
            }
            Destination::Playlists => {
                let Some(playlist) = self.catalog.playlists().get(self.current.selection) else {
                    return;
                };
                Destination::PlaylistDetails {
                    origin: PlaylistDetailsOrigin::MyPlaylists,
                    playlist_id: playlist.id().clone(),
                }
            }
            Destination::Services => {
                let Some(service) = self.catalog.services().get(self.current.selection) else {
                    return;
                };
                Destination::ServiceCatalog(service.id().clone())
            }
            Destination::ServiceCatalog(service_id) => {
                let service_catalog = self.catalog.service_catalog(&service_id);
                let Some(item) = service_catalog.items.get(self.current.selection) else {
                    return;
                };
                self.service_catalog_item_projection(&service_id, *item)
                    .details_destination
            }
            Destination::Search => {
                let results = self.catalog.search(&self.current.query);
                let Some(result) = results.items.get(self.current.selection).copied() else {
                    return;
                };
                self.search_result_projection(result).details_destination
            }
            Destination::NowPlaying => {
                let Some(PlaybackSession::Track { queue, .. }) = &self.playback else {
                    return;
                };
                let Some(track_id) = queue.get(self.current.selection) else {
                    return;
                };
                Destination::TrackDetails {
                    origin: TrackDetailsOrigin::Queue,
                    track_id: track_id.clone(),
                }
            }
            Destination::StationDetails { .. }
            | Destination::PlaylistDetails { .. }
            | Destination::TrackDetails { .. } => return,
        };

        self.history.push(self.current.clone());
        self.current = DestinationSnapshot::new(destination);
    }

    fn open_now_playing(&mut self) {
        if self.current.destination == Destination::NowPlaying {
            return;
        }
        self.history.push(self.current.clone());
        self.current = DestinationSnapshot::new(Destination::NowPlaying);
    }

    fn open_or_focus_search(&mut self) {
        if self.current.destination == Destination::Search {
            self.current.query_focused = true;
            return;
        }
        self.history.push(self.current.clone());
        self.current = DestinationSnapshot::new(Destination::Search);
    }

    fn restore_previous_destination(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.current = previous;
        }
    }

    #[must_use]
    pub fn render(&self) -> Buffer {
        let area = Rect::new(0, 0, self.viewport.width, self.viewport.height);
        let mut buffer = Buffer::empty(area);
        let base = if self.options.color_enabled {
            Style::default()
                .fg(Color::Rgb(222, 216, 202))
                .bg(Color::Rgb(27, 29, 28))
        } else {
            Style::default()
        };
        buffer.set_style(area, base);

        if !self.viewport.is_supported() {
            self.render_minimum_size_guard(&mut buffer, base);
            return buffer;
        }

        if self.help_visible {
            self.render_help(&mut buffer, base);
        } else {
            self.render_destination(&mut buffer, base);
            self.render_wide_detail_lens(&mut buffer, base);
        }
        self.render_brief_feedback(&mut buffer, base);

        buffer
    }

    fn render_wide_detail_lens(&self, buffer: &mut Buffer, base: Style) {
        let Some(content) = self.detail_lens_content() else {
            return;
        };
        let Some(separator_column) = self.wide_detail_separator_column() else {
            return;
        };
        let top = 1;
        let height = self.now_playing_top().saturating_sub(top);
        let detail_area = Rect::new(
            separator_column,
            top,
            self.viewport.width.saturating_sub(separator_column),
            height,
        );
        buffer.set_style(detail_area, base);
        let blank = " ".repeat(usize::from(detail_area.width));
        for row in top..self.now_playing_top() {
            buffer.set_string(separator_column, row, &blank, base);
            buffer.set_string(separator_column, row, "│", base);
        }

        let text_column = separator_column + 2;
        let text_width = self.viewport.width.saturating_sub(text_column + 1);
        Self::render_detail_line(
            buffer,
            text_column,
            2,
            text_width,
            "DETAIL LENS / READ ONLY",
            base,
        );
        Self::render_detail_line(buffer, text_column, 4, text_width, content.kind, base);
        Self::render_detail_line(buffer, text_column, 5, text_width, &content.title, base);
        for (index, fact) in content.facts.iter().enumerate() {
            let row = 7 + index as u16;
            if row >= self.now_playing_top() {
                break;
            }
            Self::render_detail_line(buffer, text_column, row, text_width, fact, base);
        }
    }

    fn render_detail_line(
        buffer: &mut Buffer,
        column: u16,
        row: u16,
        width: u16,
        text: &str,
        style: Style,
    ) {
        buffer.set_string(
            column,
            row,
            truncate_with_ellipsis(text, usize::from(width)),
            style,
        );
    }

    fn detail_lens_content(&self) -> Option<DetailLensContent> {
        match &self.current.destination {
            Destination::Home => {
                let selected = self.selected_home_choice().text();
                Some(DetailLensContent {
                    kind: "DESTINATION",
                    title: selected.label.to_owned(),
                    facts: vec![
                        selected.detail.to_owned(),
                        String::new(),
                        "Enter still opens the same Destination.".to_owned(),
                    ],
                })
            }
            Destination::ListeningIntents => {
                let Some(intent) = self.catalog.listening_intents().get(self.current.selection)
                else {
                    return Some(Self::empty_lens(
                        "EMPTY",
                        "No Listening intents",
                        "Nothing is available in this catalog.",
                        "Return Home and choose another path.",
                    ));
                };
                Some(DetailLensContent {
                    kind: "LISTENING INTENT",
                    title: intent.name().to_owned(),
                    facts: vec![
                        format!(
                            "{} Stations • {} Playlists",
                            intent.station_ids().len(),
                            intent.playlist_ids().len()
                        ),
                        "Source  Songdial curation".to_owned(),
                        String::new(),
                        intent.description().to_owned(),
                        String::new(),
                        "Enter opens this listening direction.".to_owned(),
                    ],
                })
            }
            Destination::Stations => {
                let Some(station) = self.catalog.stations().get(self.current.selection) else {
                    return Some(Self::empty_lens(
                        "EMPTY",
                        "No Stations",
                        "No continuous Stations are available.",
                        "Return Home and choose another path.",
                    ));
                };
                Some(self.station_lens("STATION", station))
            }
            Destination::Playlists => {
                let Some(playlist) = self.catalog.playlists().get(self.current.selection) else {
                    return Some(Self::empty_lens(
                        "EMPTY",
                        "No Playlists",
                        "No personal or saved Playlists are available.",
                        "Return Home and choose another path.",
                    ));
                };
                Some(self.playlist_lens("PLAYLIST", playlist))
            }
            Destination::Services => {
                let Some(service) = self.catalog.services().get(self.current.selection) else {
                    return Some(Self::empty_lens(
                        "EMPTY",
                        "No Services",
                        "No Service catalogs are available.",
                        "Return Home and choose another path.",
                    ));
                };
                let counts = self.catalog.service_catalog(service.id()).counts;
                Some(DetailLensContent {
                    kind: "SERVICE",
                    title: service.name().to_owned(),
                    facts: vec![
                        format!("Source  [{}]", service.badge()),
                        format!("{} Stations", counts.stations),
                        format!("{} Playlists", counts.playlists),
                        format!("{} Tracks", counts.tracks),
                        String::new(),
                        "Enter keeps Songdial navigation and keys.".to_owned(),
                    ],
                })
            }
            Destination::ListeningIntent(intent_id) => {
                let matches = self.catalog.intent_matches(intent_id);
                let Some(intent_match) = matches.get(self.current.selection) else {
                    let intent = self.catalog.listening_intent(intent_id)?;
                    return Some(Self::empty_lens(
                        "LISTENING INTENT",
                        intent.name(),
                        "No matches in the Demo catalog.",
                        "Return to Mood & activity and choose another direction.",
                    ));
                };
                match intent_match {
                    IntentMatch::Station(station) => Some(self.station_lens("STATION", station)),
                    IntentMatch::Playlist(playlist) => {
                        Some(self.playlist_lens("PLAYLIST", playlist))
                    }
                }
            }
            Destination::ServiceCatalog(service_id) => {
                let catalog = self.catalog.service_catalog(service_id);
                let Some(item) = catalog.items.get(self.current.selection) else {
                    let service = self.catalog.service(service_id)?;
                    return Some(Self::empty_lens(
                        "EMPTY",
                        service.name(),
                        "No catalog items are available.",
                        "Return to Browse services and choose another Source.",
                    ));
                };
                match item {
                    ServiceCatalogItem::Station(station) => {
                        Some(self.station_lens("STATION", station))
                    }
                    ServiceCatalogItem::Playlist(playlist) => {
                        Some(self.playlist_lens("PLAYLIST", playlist))
                    }
                    ServiceCatalogItem::Track(track) => Some(self.track_lens("TRACK", track)),
                }
            }
            Destination::Search => {
                let results = self.catalog.search(&self.current.query);
                let Some(result) = results.items.get(self.current.selection) else {
                    return if self.current.query.trim().is_empty() {
                        Some(Self::empty_lens(
                            "SEARCH",
                            "Start typing",
                            "Search every Source in the Demo catalog.",
                            "Type a query to reveal grouped results.",
                        ))
                    } else {
                        Some(Self::empty_lens(
                            "EMPTY",
                            "No results",
                            &format!("Nothing matched “{}”.", self.current.query),
                            "Edit the query to try another title, creator, or Source.",
                        ))
                    };
                };
                match result {
                    SearchResult::ListeningIntent(intent) => Some(DetailLensContent {
                        kind: "LISTENING INTENT",
                        title: intent.name().to_owned(),
                        facts: vec![
                            "Source  Songdial curation".to_owned(),
                            format!(
                                "{} Stations • {} Playlists",
                                intent.station_ids().len(),
                                intent.playlist_ids().len()
                            ),
                            String::new(),
                            intent.description().to_owned(),
                            String::new(),
                            "Enter opens without playing.".to_owned(),
                        ],
                    }),
                    SearchResult::Station(station) => Some(self.station_lens("STATION", station)),
                    SearchResult::Playlist(playlist) => {
                        Some(self.playlist_lens("PLAYLIST", playlist))
                    }
                    SearchResult::Track(track) => Some(self.track_lens("TRACK", track)),
                }
            }
            Destination::PlaylistDetails { playlist_id, .. } => {
                let tracks = self.catalog.playlist_tracks(playlist_id);
                if let Some(track) = tracks.get(self.current.selection) {
                    let mut lens = self.track_lens("TRACK", track);
                    if let Some(action) = lens.facts.last_mut() {
                        *action = match track.availability() {
                            Availability::Available => {
                                "p plays this Track • a adds it to Queue.".to_owned()
                            }
                            Availability::Loading => {
                                "p and a are unavailable while this Track is LOADING.".to_owned()
                            }
                            Availability::Unavailable(_) => {
                                "p and a are unavailable for this Track.".to_owned()
                            }
                        };
                    }
                    Some(lens)
                } else {
                    self.catalog
                        .playlist(playlist_id)
                        .map(|playlist| self.playlist_lens("PLAYLIST", playlist))
                }
            }
            Destination::NowPlaying => {
                if let Some(PlaybackSession::Track { queue, .. }) = &self.playback
                    && let Some(track_id) = queue.get(self.current.selection)
                    && let Some(track) = self.catalog.track(track_id)
                {
                    return Some(self.track_lens("QUEUED TRACK", track));
                }
                match &self.playback {
                    Some(PlaybackSession::Station { station_id, state }) => {
                        self.catalog.station(station_id).map(|station| {
                            let mut lens = self.station_lens("CURRENT STATION", station);
                            lens.facts.insert(
                                3,
                                format!(
                                    "Playback  {}",
                                    match state {
                                        StationPlaybackState::Live => "LIVE",
                                        StationPlaybackState::Paused => "LIVE • PAUSED",
                                    }
                                ),
                            );
                            lens
                        })
                    }
                    Some(PlaybackSession::Track {
                        current_track_id,
                        state,
                        ..
                    }) => self.catalog.track(current_track_id).map(|track| {
                        let mut lens = self.track_lens("CURRENT TRACK", track);
                        lens.facts.insert(
                            4,
                            format!(
                                "Playback  {}",
                                match state {
                                    TrackPlaybackState::Playing => "PLAYING",
                                    TrackPlaybackState::Paused => "PAUSED",
                                    TrackPlaybackState::Stopped => "STOPPED",
                                }
                            ),
                        );
                        lens
                    }),
                    None => Some(DetailLensContent {
                        kind: "QUEUE",
                        title: "Nothing playing".to_owned(),
                        facts: vec![
                            "Queue empty".to_owned(),
                            String::new(),
                            "Start a Track or Playlist, then add Tracks.".to_owned(),
                        ],
                    }),
                }
            }
            Destination::StationDetails { .. } | Destination::TrackDetails { .. } => None,
        }
    }

    fn station_lens(&self, kind: &'static str, station: &Station) -> DetailLensContent {
        let mut facts = vec![
            format!("Style  {}", station.style()),
            format!(
                "Source  {} [{}]",
                self.catalog.source_name(station.source_id()),
                self.catalog.source_badge(station.source_id())
            ),
            format!(
                "Status  {}",
                Self::availability_label(station.availability())
            ),
        ];
        if let Availability::Unavailable(reason) = station.availability() {
            facts.push(reason.clone());
        }
        facts.extend([
            String::new(),
            station.description().to_owned(),
            String::new(),
            "Enter opens details without playing.".to_owned(),
        ]);
        DetailLensContent {
            kind,
            title: station.name().to_owned(),
            facts,
        }
    }

    fn empty_lens(
        kind: &'static str,
        title: &str,
        explanation: &str,
        recovery: &str,
    ) -> DetailLensContent {
        DetailLensContent {
            kind,
            title: title.to_owned(),
            facts: vec![explanation.to_owned(), String::new(), recovery.to_owned()],
        }
    }

    fn playlist_lens(&self, kind: &'static str, playlist: &Playlist) -> DetailLensContent {
        DetailLensContent {
            kind,
            title: playlist.name().to_owned(),
            facts: vec![
                format!("{} Tracks", playlist.track_ids().len()),
                format!(
                    "Source  {} [{}]",
                    self.catalog.source_name(playlist.source_id()),
                    self.catalog.source_badge(playlist.source_id())
                ),
                format!(
                    "Status  {}",
                    if playlist.track_ids().is_empty() {
                        "EMPTY"
                    } else {
                        Self::availability_label(playlist.availability())
                    }
                ),
                String::new(),
                playlist.description().to_owned(),
                String::new(),
                "Enter opens Tracks without playing.".to_owned(),
            ],
        }
    }

    fn track_lens(&self, kind: &'static str, track: &Track) -> DetailLensContent {
        let mut facts = vec![
            format!("Creator  {}", track.creator()),
            format!(
                "Source  {} [{}]",
                self.catalog.source_name(track.source_id()),
                self.catalog.source_badge(track.source_id())
            ),
            format!(
                "Duration  {}",
                Self::format_duration(track.duration_seconds())
            ),
            format!("Status  {}", Self::availability_label(track.availability())),
        ];
        if let Availability::Unavailable(reason) = track.availability() {
            facts.push(reason.clone());
        }
        facts.extend([
            String::new(),
            "Enter opens details without playing.".to_owned(),
        ]);
        DetailLensContent {
            kind,
            title: track.name().to_owned(),
            facts,
        }
    }

    fn wide_detail_separator_column(&self) -> Option<u16> {
        const OUTER_GUTTERS: u16 = 4;
        const LIST_CONTENT: u16 = 56;
        const SEPARATOR: u16 = 1;
        const DETAIL_CONTENT: u16 = 40;

        let usable_content_width = self.viewport.width.saturating_sub(OUTER_GUTTERS);
        (usable_content_width >= LIST_CONTENT + SEPARATOR + DETAIL_CONTENT)
            .then_some(LIST_CONTENT + 2)
    }

    fn render_minimum_size_guard(&self, buffer: &mut Buffer, base: Style) {
        let start_row = self.viewport.height.saturating_sub(7) / 2;
        let lines = [
            (0, "SONGDIAL NEEDS MORE ROOM".to_owned()),
            (
                2,
                format!(
                    "Current  {}×{} cells",
                    self.viewport.width, self.viewport.height
                ),
            ),
            (3, "Required 80×24 cells".to_owned()),
            (5, "Resize to recover the unchanged session.".to_owned()),
            (6, "q quit".to_owned()),
        ];

        for (offset, line) in lines {
            let row = start_row.saturating_add(offset);
            if row >= self.viewport.height {
                continue;
            }
            let width = u16::try_from(line.chars().count()).unwrap_or(u16::MAX);
            let column = self.viewport.width.saturating_sub(width) / 2;
            buffer.set_string(column, row, line, base);
        }
    }

    fn render_brief_feedback(&self, buffer: &mut Buffer, base: Style) {
        let Some(PlaybackFeedback::Brief(message)) = &self.playback_feedback else {
            return;
        };
        let blank = " ".repeat(usize::from(self.viewport.width));
        buffer.set_string(0, self.guide_top(), &blank, base);
        buffer.set_string(0, self.guide_top() + 1, &blank, base);
        buffer.set_string(0, self.guide_top(), format!(" QUEUE  {message}"), base);
    }

    fn render_destination(&self, buffer: &mut Buffer, base: Style) {
        match &self.current.destination {
            Destination::Home => self.render_home(buffer, base),
            Destination::ListeningIntents => self.render_listening_intents(buffer, base),
            Destination::ListeningIntent(intent_id) => {
                self.render_listening_intent(buffer, base, intent_id);
            }
            Destination::Stations => self.render_stations(buffer, base),
            Destination::Playlists => self.render_playlists(buffer, base),
            Destination::Services => self.render_services(buffer, base),
            Destination::ServiceCatalog(service_id) => {
                self.render_service_catalog(buffer, base, service_id);
            }
            Destination::Search => self.render_search(buffer, base),
            Destination::StationDetails { origin, station_id } => {
                self.render_station_details(buffer, base, origin, station_id)
            }
            Destination::PlaylistDetails {
                origin,
                playlist_id,
            } => self.render_playlist_details(buffer, base, origin, playlist_id),
            Destination::NowPlaying => self.render_now_playing_destination(buffer, base),
            Destination::TrackDetails { origin, track_id } => {
                self.render_track_details(buffer, base, origin, track_id);
            }
        }
    }

    fn render_location(&self, buffer: &mut Buffer, location: &str, base: Style) {
        buffer.set_string(
            0,
            0,
            truncate_middle_with_ellipsis(location, usize::from(self.viewport.width)),
            base,
        );
    }

    fn render_home(&self, buffer: &mut Buffer, base: Style) {
        self.render_location(buffer, " SONGDIAL / HOME", base);
        buffer.set_string(0, 2, "  CHOOSE WHAT FITS RIGHT NOW", base);
        for (index, choice) in HomeChoice::ALL.iter().enumerate() {
            let label = choice.text().label;
            let line = if index == self.current.selection {
                format!("  > {label:<58}SELECTED")
            } else {
                format!("    {label}")
            };
            buffer.set_string(0, 4 + index as u16, line, base);
        }
        buffer.set_string(
            0,
            10,
            "  Enter opens a Destination. Playback always starts separately.",
            base,
        );
        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " ↑/k up  ↓/j down  Enter open", base);
        buffer.set_string(0, self.guide_top() + 1, " n queue  ? help  q quit", base);

        let selected = self.selected_style();
        if self.current.active_pane == ActivePane::List {
            buffer.set_style(
                self.dense_list_selection_area(4 + self.current.selection as u16, 1),
                selected,
            );
        }
    }

    fn render_listening_intents(&self, buffer: &mut Buffer, base: Style) {
        self.render_location(buffer, " SONGDIAL / MOOD & ACTIVITY", base);
        buffer.set_string(0, 2, "  MOOD & ACTIVITY", base);
        buffer.set_string(0, 3, "  Choose what fits right now.", base);

        if self.catalog.listening_intents().is_empty() {
            buffer.set_string(0, 4, "  0 Listening intents • EMPTY", base);
            buffer.set_string(0, 7, "  No Listening intents are available.", base);
            buffer.set_string(0, 8, "  Esc returns Home to choose another path.", base);
        }

        for (index, intent) in self.catalog.listening_intents().iter().enumerate() {
            let line = if index == self.current.selection {
                format!("  > {:<58}SELECTED", intent.name())
            } else {
                format!("    {}", intent.name())
            };
            buffer.set_string(0, 5 + index as u16, line, base);
        }

        self.render_now_playing(buffer, base);
        if self.catalog.listening_intents().is_empty() {
            buffer.set_string(
                0,
                self.guide_top(),
                " n queue  Esc back  ? help  q quit",
                base,
            );
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k up  ↓/j down  Enter open", base);
            buffer.set_string(0, self.guide_top() + 1, " n queue  Esc back", base);
        }

        let selected = self.selected_style();
        if !self.catalog.listening_intents().is_empty()
            && self.current.active_pane == ActivePane::List
        {
            buffer.set_style(
                self.dense_list_selection_area(5 + self.current.selection as u16, 1),
                selected,
            );
        }
    }

    fn render_listening_intent(&self, buffer: &mut Buffer, base: Style, intent_id: &CatalogId) {
        let Some(intent) = self.catalog.listening_intent(intent_id) else {
            return;
        };
        let matches = self.catalog.intent_matches(intent_id);

        self.render_location(
            buffer,
            &format!(
                " SONGDIAL / MOOD & ACTIVITY / {}",
                intent.name().to_uppercase()
            ),
            base,
        );
        buffer.set_string(0, 2, format!("  {}", intent.name().to_uppercase()), base);
        buffer.set_string(0, 3, format!("  {}", intent.description()), base);
        let counts = format!(
            "  {} Stations • {} Playlists",
            intent.station_ids().len(),
            intent.playlist_ids().len()
        );
        if matches.is_empty() {
            buffer.set_string(0, 4, format!("{counts} • EMPTY"), base);
            buffer.set_string(
                0,
                7,
                format!("  Nothing matches {} in the Demo catalog.", intent.name()),
                base,
            );
            buffer.set_string(
                0,
                8,
                "  Esc returns to Mood & activity to choose another direction.",
                base,
            );
        } else {
            buffer.set_string(
                0,
                4,
                format!(
                    "{counts} • Choice {}/{}",
                    self.current.selection + 1,
                    matches.len()
                ),
                base,
            );
        }

        for (slot, intent_match) in matches
            .iter()
            .skip(self.current.scroll_offset)
            .take(self.dense_visible_items())
            .enumerate()
        {
            let index = self.current.scroll_offset + slot;
            let row = 6 + (slot as u16 * 2);
            let selected = index == self.current.selection;
            let (kind, title, source_id, detail, availability, playing) = match intent_match {
                IntentMatch::Station(station) => (
                    "STATION",
                    station.name(),
                    station.source_id(),
                    station.style(),
                    station.availability(),
                    matches!(
                        &self.playback,
                        Some(PlaybackSession::Station { station_id, .. })
                            if station_id == station.id()
                    ),
                ),
                IntentMatch::Playlist(playlist) => (
                    "PLAYLIST",
                    playlist.name(),
                    playlist.source_id(),
                    if playlist.track_ids().is_empty() {
                        "EMPTY"
                    } else {
                        "ORDERED TRACKS"
                    },
                    playlist.availability(),
                    self.playlist_playback_is_active(playlist.id()),
                ),
            };
            self.render_dense_catalog_row(
                buffer,
                base,
                DenseCatalogRow {
                    row,
                    selected,
                    playing,
                    kind,
                    title,
                    source_id,
                    detail,
                    availability,
                    status: Self::availability_label(availability),
                },
            );
        }

        self.render_now_playing(buffer, base);
        if matches.is_empty() {
            buffer.set_string(0, self.guide_top(), " Esc back  ? help  q quit", base);
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  Enter inspect", base);
            buffer.set_string(
                0,
                self.guide_top() + 1,
                match matches.get(self.current.selection) {
                    Some(IntentMatch::Playlist(_)) => " p play  a add  Esc back",
                    Some(IntentMatch::Station(_)) | None => " p play  n queue  Esc back",
                },
                base,
            );
        }
    }

    fn render_stations(&self, buffer: &mut Buffer, base: Style) {
        let stations = self.catalog.stations();
        self.render_location(buffer, " SONGDIAL / RADIO STATIONS", base);
        buffer.set_string(0, 2, "  RADIO STATIONS", base);
        buffer.set_string(0, 3, "  Continuous music from every Source.", base);
        if stations.is_empty() {
            buffer.set_string(0, 4, "  0 Stations • EMPTY", base);
            buffer.set_string(0, 7, "  No Stations are available.", base);
            buffer.set_string(0, 8, "  Esc returns Home to choose another path.", base);
        } else {
            buffer.set_string(
                0,
                4,
                format!(
                    "  {} Stations • Station {}/{}",
                    stations.len(),
                    self.current.selection + 1,
                    stations.len()
                ),
                base,
            );
        }

        for (slot, station) in stations
            .iter()
            .skip(self.current.scroll_offset)
            .take(self.dense_visible_items())
            .enumerate()
        {
            let index = self.current.scroll_offset + slot;
            let selected = index == self.current.selection;
            let playing = matches!(
                &self.playback,
                Some(PlaybackSession::Station { station_id, .. }) if station_id == station.id()
            );
            let row = 6 + (slot as u16 * 2);
            self.render_dense_catalog_row(
                buffer,
                base,
                DenseCatalogRow {
                    row,
                    selected,
                    playing,
                    kind: "STATION",
                    title: station.name(),
                    source_id: station.source_id(),
                    detail: station.style(),
                    availability: station.availability(),
                    status: Self::availability_label(station.availability()),
                },
            );
        }

        self.render_now_playing(buffer, base);
        if stations.is_empty() {
            buffer.set_string(
                0,
                self.guide_top(),
                " n queue  Esc back  ? help  q quit",
                base,
            );
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  Enter inspect", base);
            let selected_is_unavailable =
                stations.get(self.current.selection).is_some_and(|station| {
                    matches!(station.availability(), Availability::Unavailable(_))
                });
            buffer.set_string(
                0,
                self.guide_top() + 1,
                if selected_is_unavailable {
                    " p unavailable  n queue  Esc back"
                } else {
                    " p play  n queue  Esc back"
                },
                base,
            );
        }
    }

    fn render_playlists(&self, buffer: &mut Buffer, base: Style) {
        let playlists = self.catalog.playlists();
        self.render_location(buffer, " SONGDIAL / MY PLAYLISTS", base);
        buffer.set_string(0, 2, "  MY PLAYLISTS", base);
        buffer.set_string(
            0,
            3,
            "  Personal and saved Playlists from every Source.",
            base,
        );
        if playlists.is_empty() {
            buffer.set_string(0, 4, "  0 Playlists • EMPTY", base);
            buffer.set_string(0, 7, "  No Playlists are saved in the Demo catalog.", base);
            buffer.set_string(
                0,
                8,
                "  Esc returns Home to choose another listening path.",
                base,
            );
        } else {
            buffer.set_string(
                0,
                4,
                format!(
                    "  {} Playlists • Playlist {}/{}",
                    playlists.len(),
                    self.current.selection + 1,
                    playlists.len()
                ),
                base,
            );
        }

        for (slot, playlist) in playlists
            .iter()
            .skip(self.current.scroll_offset)
            .take(self.dense_visible_items())
            .enumerate()
        {
            let index = self.current.scroll_offset + slot;
            let selected = index == self.current.selection;
            let playing = self.playlist_playback_is_active(playlist.id());
            let track_count = playlist.track_ids().len();
            let track_suffix = if track_count == 1 { "" } else { "s" };
            let status = if playlist.track_ids().is_empty() {
                "EMPTY"
            } else {
                Self::availability_label(playlist.availability())
            };
            let detail = format!("{track_count} Track{track_suffix}");
            let row = 6 + (slot as u16 * 2);
            self.render_dense_catalog_row(
                buffer,
                base,
                DenseCatalogRow {
                    row,
                    selected,
                    playing,
                    kind: "PLAYLIST",
                    title: playlist.name(),
                    source_id: playlist.source_id(),
                    detail: &detail,
                    availability: playlist.availability(),
                    status,
                },
            );
        }

        self.render_now_playing(buffer, base);
        if playlists.is_empty() {
            buffer.set_string(
                0,
                self.guide_top(),
                " n queue  Esc back  ? help  q quit",
                base,
            );
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  Enter inspect", base);
            let selected_is_empty = playlists
                .get(self.current.selection)
                .is_some_and(|playlist| playlist.track_ids().is_empty());
            buffer.set_string(
                0,
                self.guide_top() + 1,
                if selected_is_empty {
                    " p unavailable  a unavailable  Esc back"
                } else {
                    " p play  a add  Esc back"
                },
                base,
            );
        }
    }

    fn render_services(&self, buffer: &mut Buffer, base: Style) {
        let services = self.catalog.services();
        self.render_location(buffer, " SONGDIAL / BROWSE SERVICES", base);
        buffer.set_string(0, 2, "  BROWSE SERVICES", base);
        buffer.set_string(
            0,
            3,
            "  Choose a fictional Service catalog by Source.",
            base,
        );
        if services.is_empty() {
            buffer.set_string(0, 4, "  0 Services • EMPTY", base);
            buffer.set_string(0, 7, "  No Services are available.", base);
            buffer.set_string(0, 8, "  Esc returns Home to choose another path.", base);
        } else {
            buffer.set_string(
                0,
                4,
                format!(
                    "  {} Services • Service {}/{}",
                    services.len(),
                    self.current.selection + 1,
                    services.len()
                ),
                base,
            );
        }

        for (index, service) in services.iter().enumerate() {
            let selected = index == self.current.selection;
            let state = Self::dense_row_state(selected, false, &Availability::Available);
            let counts = self.catalog.service_catalog(service.id()).counts;
            let row = 6 + index as u16 * 2;
            buffer.set_string(
                0,
                row,
                dense_catalog_line(
                    self.browser_list_width(),
                    state,
                    "SERVICE",
                    service.name(),
                    "CATALOG",
                    service.badge(),
                ),
                base,
            );
            buffer.set_string(
                0,
                row + 1,
                format!(
                    "             {} Stations • {} Playlists • {} Tracks",
                    counts.stations, counts.playlists, counts.tracks
                ),
                base,
            );
            if selected && self.current.active_pane == ActivePane::List {
                buffer.set_style(
                    self.dense_list_selection_area(row, 2),
                    self.selected_style(),
                );
            }
        }

        self.render_now_playing(buffer, base);
        if services.is_empty() {
            buffer.set_string(
                0,
                self.guide_top(),
                " n queue  Esc back  ? help  q quit",
                base,
            );
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  Enter open", base);
            buffer.set_string(0, self.guide_top() + 1, " n queue  Esc back  ? help", base);
        }
    }

    fn render_search(&self, buffer: &mut Buffer, base: Style) {
        self.render_location(buffer, " SONGDIAL / SEARCH EVERYTHING", base);
        buffer.set_string(0, 2, "  SEARCH EVERYTHING", base);
        buffer.set_string(
            0,
            3,
            format!(
                "  Query {} {}",
                if self.current.query_focused { '>' } else { ' ' },
                self.current.query
            ),
            base,
        );
        let results = self.catalog.search(&self.current.query);
        if self.current.query.trim().is_empty() {
            buffer.set_string(0, 4, "  Start typing to search the Demo catalog.", base);
        } else if results.items.is_empty() {
            buffer.set_string(
                0,
                4,
                format!("  No results for “{}”.", self.current.query),
                base,
            );
        } else {
            buffer.set_string(
                0,
                4,
                format!(
                    "  {} result{} • Result {}/{}",
                    results.items.len(),
                    if results.items.len() == 1 { "" } else { "s" },
                    self.current.selection + 1,
                    results.items.len()
                ),
                base,
            );

            let visible = results
                .items
                .iter()
                .skip(self.current.scroll_offset)
                .take(self.search_visible_items());
            let mut row = 6;
            let mut previous_group = None;
            for (slot, result) in visible.enumerate() {
                let group = result.group();
                if previous_group != Some(group) {
                    let group_count = results
                        .items
                        .iter()
                        .filter(|item| item.group() == group)
                        .count();
                    buffer.set_string(0, row, format!("  {} • {group_count}", group.label()), base);
                    row += 1;
                    previous_group = Some(group);
                }
                let index = self.current.scroll_offset + slot;
                self.render_search_result(
                    buffer,
                    base,
                    row,
                    index == self.current.selection,
                    *result,
                );
                row += 1;
            }
        }
        self.render_now_playing(buffer, base);
        if self.current.query_focused {
            buffer.set_string(
                0,
                self.guide_top(),
                " Type to search  Backspace erase",
                base,
            );
            buffer.set_string(
                0,
                self.guide_top() + 1,
                " ↑/↓ move  Enter inspect  Esc done",
                base,
            );
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  Enter inspect", base);
            let actions = results.items.get(self.current.selection).copied().map_or(
                " / edit query  n queue  Esc back",
                |result| {
                    let projection = self.search_result_projection(result);
                    match (
                        projection.play_is_applicable(),
                        projection.can_play(),
                        projection.queue_is_applicable,
                        projection.can_queue(),
                    ) {
                        (false, _, _, _) => " / edit query  n queue  Esc back",
                        (true, false, false, _) => " p unavailable  / edit  Esc back",
                        (true, true, false, _) => " p play  / edit query  Esc back",
                        (_, false, true, _) => " p unavailable  a unavailable  Esc back",
                        (_, true, true, false) => " p play  a unavailable  Esc back",
                        (_, true, true, true) => " p play  a add  / edit  Esc back",
                    }
                },
            );
            buffer.set_string(0, self.guide_top() + 1, actions, base);
        }
    }

    fn render_search_result(
        &self,
        buffer: &mut Buffer,
        base: Style,
        row: u16,
        selected: bool,
        result: SearchResult<'_>,
    ) {
        let projection = self.search_result_projection(result);
        let state = Self::dense_row_state(selected, projection.playing, projection.availability);
        let source = projection
            .source_id
            .map(|source_id| self.catalog.source_badge(source_id))
            .unwrap_or("SONGDIAL");
        let status = match result {
            SearchResult::ListeningIntent(_) => "CURATED".to_owned(),
            SearchResult::Station(station) => {
                Self::availability_label(station.availability()).to_owned()
            }
            SearchResult::Playlist(playlist) if playlist.track_ids().is_empty() => {
                "EMPTY".to_owned()
            }
            SearchResult::Playlist(playlist) => {
                Self::availability_label(playlist.availability()).to_owned()
            }
            SearchResult::Track(track) => Self::track_row_status(track),
        };
        buffer.set_string(
            0,
            row,
            dense_catalog_line(
                self.browser_list_width(),
                state,
                projection.kind,
                projection.title,
                &status,
                source,
            ),
            base,
        );
        if selected {
            buffer.set_style(
                self.dense_list_selection_area(row, 1),
                self.selected_style(),
            );
        }
    }

    fn render_service_catalog(&self, buffer: &mut Buffer, base: Style, service_id: &CatalogId) {
        let service = self
            .catalog
            .service(service_id)
            .expect("Service catalog should reference a Demo Service");
        let service_catalog = self.catalog.service_catalog(service_id);
        let items = service_catalog.items;
        let counts = service_catalog.counts;
        self.render_location(
            buffer,
            &format!(
                " SONGDIAL / BROWSE SERVICES / {}",
                service.name().to_uppercase()
            ),
            base,
        );
        buffer.set_string(0, 2, format!("  {}", service.name().to_uppercase()), base);
        buffer.set_string(
            0,
            3,
            "  Source-filtered Stations, Playlists, and Tracks.",
            base,
        );
        if items.is_empty() {
            buffer.set_string(
                0,
                4,
                format!(
                    "  {} Stations • {} Playlists • {} Tracks • EMPTY",
                    counts.stations, counts.playlists, counts.tracks
                ),
                base,
            );
            buffer.set_string(0, 7, "  No catalog items are available.", base);
            buffer.set_string(
                0,
                8,
                "  Esc returns to Browse services to choose another Source.",
                base,
            );
        } else {
            buffer.set_string(
                0,
                4,
                format!(
                    "  {} Stations • {} Playlists • {} Tracks • Item {}/{}",
                    counts.stations,
                    counts.playlists,
                    counts.tracks,
                    self.current.selection + 1,
                    items.len()
                ),
                base,
            );
        }

        for (slot, item) in items
            .iter()
            .skip(self.current.scroll_offset)
            .take(self.dense_visible_items())
            .enumerate()
        {
            let index = self.current.scroll_offset + slot;
            let selected = index == self.current.selection;
            let row = 6 + slot as u16 * 2;
            let projection = self.service_catalog_item_projection(service_id, *item);
            self.render_dense_catalog_row(
                buffer,
                base,
                DenseCatalogRow {
                    row,
                    selected,
                    playing: projection.playing,
                    kind: projection.kind,
                    title: projection.title,
                    source_id: projection.source_id,
                    detail: &projection.detail,
                    availability: projection.availability,
                    status: &projection.status,
                },
            );
        }

        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  Enter inspect", base);
        let actions = match items
            .get(self.current.selection)
            .copied()
            .map(|item| self.service_catalog_item_projection(service_id, item))
        {
            Some(item) if !item.queue_is_applicable && item.can_play() => {
                " p play  n queue  Esc back"
            }
            Some(item) if !item.queue_is_applicable => " p unavailable  n queue  Esc back",
            Some(item) if item.can_play() && item.can_queue() => " p play  a add  Esc back",
            Some(_) => " p unavailable  a unavailable  Esc back",
            None => " Esc back  ? help  q quit",
        };
        buffer.set_string(0, self.guide_top() + 1, actions, base);
    }

    fn render_station_details(
        &self,
        buffer: &mut Buffer,
        base: Style,
        origin: &StationDetailsOrigin,
        station_id: &CatalogId,
    ) {
        let parent = match origin {
            StationDetailsOrigin::ListeningIntent(intent_id) => {
                let intent = self
                    .catalog
                    .listening_intent(intent_id)
                    .expect("fixed Station parent intent should exist");
                format!("MOOD & ACTIVITY / {}", intent.name().to_uppercase())
            }
            StationDetailsOrigin::RadioStations => "RADIO STATIONS".to_owned(),
            StationDetailsOrigin::ServiceCatalog(service_id) => {
                self.service_catalog_breadcrumb(service_id)
            }
            StationDetailsOrigin::Search => "SEARCH EVERYTHING".to_owned(),
        };
        let station = self
            .catalog
            .station(station_id)
            .expect("fixed Station should exist");
        let status = Self::availability_label(station.availability());

        self.render_location(
            buffer,
            &format!(" SONGDIAL / {parent} / {}", station.name().to_uppercase()),
            base,
        );
        buffer.set_string(0, 2, "  STATION", base);
        buffer.set_string(0, 4, format!("  {}", station.name()), base);
        buffer.set_string(0, 5, format!("  {}", station.style()), base);
        buffer.set_string(
            0,
            6,
            format!(
                "  Source  {}",
                self.catalog.source_name(station.source_id())
            ),
            base,
        );
        buffer.set_string(0, 8, format!("  {}", station.description()), base);
        buffer.set_string(0, 10, format!("  Status  {status}"), base);
        if let Availability::Unavailable(reason) = station.availability() {
            buffer.set_string(0, 11, format!("  {reason}"), base);
        }
        buffer.set_string(
            0,
            12,
            "  Enter opened details only. Nothing started playing.",
            base,
        );
        buffer.set_string(
            0,
            14,
            "  Queue  Stations are continuous and cannot be added.",
            base,
        );
        self.render_now_playing(buffer, base);
        if matches!(station.availability(), Availability::Unavailable(_)) {
            buffer.set_string(
                0,
                self.guide_top(),
                " p unavailable  n queue  Esc back",
                base,
            );
            buffer.set_string(0, self.guide_top() + 1, " ? help  q quit", base);
        } else {
            buffer.set_string(0, self.guide_top(), " p play  Space pause  n queue", base);
            buffer.set_string(0, self.guide_top() + 1, " Esc back  ? help", base);
        }
    }

    fn render_playlist_details(
        &self,
        buffer: &mut Buffer,
        base: Style,
        origin: &PlaylistDetailsOrigin,
        playlist_id: &CatalogId,
    ) {
        let (parent, return_destination) = match origin {
            PlaylistDetailsOrigin::ListeningIntent(intent_id) => {
                let intent = self
                    .catalog
                    .listening_intent(intent_id)
                    .expect("fixed Playlist parent intent should exist");
                (
                    format!("MOOD & ACTIVITY / {}", intent.name().to_uppercase()),
                    intent.name().to_owned(),
                )
            }
            PlaylistDetailsOrigin::MyPlaylists => {
                ("MY PLAYLISTS".to_owned(), "My playlists".to_owned())
            }
            PlaylistDetailsOrigin::ServiceCatalog(service_id) => (
                self.service_catalog_breadcrumb(service_id),
                self.catalog.source_name(service_id).to_owned(),
            ),
            PlaylistDetailsOrigin::Search => ("SEARCH EVERYTHING".to_owned(), "Search".to_owned()),
        };
        let playlist = self
            .catalog
            .playlist(playlist_id)
            .expect("fixed Playlist should exist");
        let tracks = self.catalog.playlist_tracks(playlist_id);

        self.render_location(
            buffer,
            &format!(" SONGDIAL / {parent} / {}", playlist.name().to_uppercase()),
            base,
        );
        buffer.set_string(0, 2, "  PLAYLIST", base);
        buffer.set_string(0, 3, format!("  {}", playlist.name()), base);
        buffer.set_string(0, 4, format!("  {}", playlist.description()), base);
        buffer.set_string(
            0,
            5,
            format!(
                "  Source  {} • {} Tracks",
                self.catalog.source_name(playlist.source_id()),
                playlist.track_ids().len()
            ),
            base,
        );
        if playlist.track_ids().is_empty() {
            buffer.set_string(0, 6, "  TRACKS • EMPTY", base);
            buffer.set_string(
                0,
                8,
                "  This Playlist has no Tracks in the Demo catalog.",
                base,
            );
            buffer.set_string(
                0,
                9,
                format!(
                    "  Esc returns to {} without changing Now Playing.",
                    return_destination
                ),
                base,
            );
        } else {
            buffer.set_string(
                0,
                6,
                format!(
                    "  TRACKS • Track {}/{}",
                    self.current.selection + 1,
                    playlist.track_ids().len()
                ),
                base,
            );
        }

        for (slot, track) in tracks
            .iter()
            .skip(self.current.scroll_offset)
            .take(self.playlist_visible_items())
            .enumerate()
        {
            let index = self.current.scroll_offset + slot;
            let selected = index == self.current.selection;
            let playing = self.track_playback_is_active(track.id());
            let state = Self::dense_row_state(selected, playing, track.availability());
            let row = 7 + slot as u16;
            let status = Self::track_row_status(track);

            buffer.set_string(
                0,
                row,
                dense_catalog_line(
                    self.browser_list_width(),
                    state,
                    "TRACK",
                    track.name(),
                    &status,
                    self.catalog.source_badge(track.source_id()),
                ),
                base,
            );

            if selected && self.current.active_pane == ActivePane::List {
                buffer.set_style(
                    self.dense_list_selection_area(row, 1),
                    self.selected_style(),
                );
            }
        }

        self.render_now_playing(buffer, base);
        if playlist.track_ids().is_empty() {
            buffer.set_string(
                0,
                self.guide_top(),
                " p unavailable  a unavailable  Esc back",
                base,
            );
            buffer.set_string(0, self.guide_top() + 1, " n queue  ? help  q quit", base);
        } else if tracks
            .get(self.current.selection)
            .is_some_and(|track| track.availability() != &Availability::Available)
        {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  p unavailable", base);
            buffer.set_string(
                0,
                self.guide_top() + 1,
                " a unavailable  n queue  Esc back",
                base,
            );
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  p play", base);
            buffer.set_string(0, self.guide_top() + 1, " a add  n queue  Esc back", base);
        }
    }

    fn render_now_playing_destination(&self, buffer: &mut Buffer, base: Style) {
        self.render_location(buffer, " SONGDIAL / NOW PLAYING", base);
        buffer.set_string(0, 2, "  NOW PLAYING & QUEUE", base);

        let current = self.playback.as_ref().map_or_else(
            || "Nothing playing".to_owned(),
            |playback| {
                self.playback_summary_line(playback)
                    .trim_start_matches(" NOW PLAYING  ")
                    .to_owned()
            },
        );
        buffer.set_string(0, 3, format!("  CURRENT  {current}"), base);

        let queue = match &self.playback {
            Some(PlaybackSession::Track { queue, .. }) => Some(queue),
            Some(PlaybackSession::Station { .. }) | None => None,
        };
        let queue_len = queue.map_or(0, Queue::len);
        if queue_len == 0 {
            buffer.set_string(0, 5, "  QUEUE • Empty", base);
            match &self.playback {
                Some(PlaybackSession::Track { .. }) => {
                    buffer.set_string(0, 7, "  Queue is empty.", base);
                    buffer.set_string(
                        0,
                        8,
                        "  Browse Tracks or Playlists and press a to add them.",
                        base,
                    );
                }
                Some(PlaybackSession::Station { .. }) => {
                    buffer.set_string(
                        0,
                        7,
                        "  Continuous Stations remain LIVE with an empty Queue.",
                        base,
                    );
                    buffer.set_string(
                        0,
                        8,
                        "  Play a Track or Playlist to replace this Station.",
                        base,
                    );
                }
                None => {
                    buffer.set_string(0, 7, "  No Playback session is active.", base);
                    buffer.set_string(
                        0,
                        8,
                        "  Start a Track or Playlist, then press a to add more Tracks.",
                        base,
                    );
                }
            }
        } else {
            let suffix = if queue_len == 1 { "" } else { "s" };
            buffer.set_string(
                0,
                5,
                format!(
                    "  QUEUE • {queue_len} Track{suffix} • Track {}/{}",
                    self.current.selection + 1,
                    queue_len
                ),
                base,
            );
        }

        if let Some(queue) = queue {
            for (slot, track_id) in queue
                .iter()
                .skip(self.current.scroll_offset)
                .take(self.queue_visible_items())
                .enumerate()
            {
                let track = self
                    .catalog
                    .track(track_id)
                    .expect("Queue should reference a catalog Track");
                let index = self.current.scroll_offset + slot;
                let selected = index == self.current.selection;
                let state = Self::dense_row_state(selected, false, track.availability());
                let row = 6 + slot as u16;
                let status = Self::track_row_status(track);
                buffer.set_string(
                    0,
                    row,
                    dense_catalog_line(
                        self.browser_list_width(),
                        state,
                        "TRACK",
                        track.name(),
                        &status,
                        self.catalog.source_badge(track.source_id()),
                    ),
                    base,
                );
                if selected && self.current.active_pane == ActivePane::List {
                    buffer.set_style(
                        self.dense_list_selection_area(row, 1),
                        self.selected_style(),
                    );
                }
            }
        }

        self.render_now_playing(buffer, base);
        if queue_len == 0 {
            buffer.set_string(
                0,
                self.guide_top(),
                if self.playback.is_some() {
                    " Space pause  Esc back  ? help  q quit"
                } else {
                    " Esc back  ? help  q quit"
                },
                base,
            );
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  Enter inspect", base);
            buffer.set_string(0, self.guide_top() + 1, " p play  d remove  Esc back", base);
        }
    }

    fn render_track_details(
        &self,
        buffer: &mut Buffer,
        base: Style,
        origin: &TrackDetailsOrigin,
        track_id: &CatalogId,
    ) {
        let track = self
            .catalog
            .track(track_id)
            .expect("Track details should reference a catalog Track");
        let parent = match origin {
            TrackDetailsOrigin::ServiceCatalog(service_id) => {
                self.service_catalog_breadcrumb(service_id)
            }
            TrackDetailsOrigin::Search => "SEARCH EVERYTHING".to_owned(),
            TrackDetailsOrigin::Queue => "NOW PLAYING".to_owned(),
        };
        let status = Self::availability_label(track.availability());
        self.render_location(
            buffer,
            &format!(" SONGDIAL / {parent} / {}", track.name().to_uppercase()),
            base,
        );
        buffer.set_string(0, 2, "  TRACK", base);
        buffer.set_string(0, 4, format!("  {}", track.name()), base);
        buffer.set_string(0, 5, format!("  {}", track.creator()), base);
        buffer.set_string(
            0,
            6,
            format!("  Source  {}", self.catalog.source_name(track.source_id())),
            base,
        );
        buffer.set_string(
            0,
            8,
            format!(
                "  Duration  {}",
                Self::format_duration(track.duration_seconds())
            ),
            base,
        );
        buffer.set_string(0, 10, format!("  Status  {status}"), base);
        if let Availability::Unavailable(reason) = track.availability() {
            buffer.set_string(0, 11, format!("  {reason}"), base);
        }
        if matches!(
            origin,
            TrackDetailsOrigin::ServiceCatalog(_) | TrackDetailsOrigin::Search
        ) {
            buffer.set_string(
                0,
                12,
                "  Enter opened details only. Nothing started playing.",
                base,
            );
        }
        self.render_now_playing(buffer, base);
        match origin {
            TrackDetailsOrigin::ServiceCatalog(_) | TrackDetailsOrigin::Search => {
                let unavailable = matches!(track.availability(), Availability::Unavailable(_));
                buffer.set_string(
                    0,
                    self.guide_top(),
                    if unavailable {
                        " p unavailable  Space pause  n queue"
                    } else {
                        " p play  Space pause  n queue"
                    },
                    base,
                );
                buffer.set_string(
                    0,
                    self.guide_top() + 1,
                    if unavailable {
                        " a unavailable  Esc back  ? help"
                    } else {
                        " a add  Esc back  ? help"
                    },
                    base,
                );
            }
            TrackDetailsOrigin::Queue => buffer.set_string(
                0,
                self.guide_top(),
                " Space pause  Esc back  ? help  q quit",
                base,
            ),
        }
    }

    fn render_now_playing(&self, buffer: &mut Buffer, base: Style) {
        let top = self.now_playing_top();
        match (&self.playback, &self.pending_playback) {
            (Some(playback), Some(pending)) => {
                buffer.set_string(0, top, self.playback_summary_line(playback), base);
                buffer.set_string(
                    0,
                    top + 1,
                    self.pending_target_line(
                        pending,
                        &format!("              {} ", self.loading_label()),
                    ),
                    base,
                );
            }
            (None, Some(pending)) => {
                buffer.set_string(
                    0,
                    top,
                    self.pending_target_line(
                        pending,
                        &format!(" NOW PLAYING  {} ", self.loading_label()),
                    ),
                    base,
                );
                buffer.set_string(
                    0,
                    top + 1,
                    "              Waiting for simulated playback.",
                    base,
                );
            }
            (Some(playback), None) => {
                buffer.set_string(0, top, self.playback_summary_line(playback), base);
                buffer.set_string(
                    0,
                    top + 1,
                    self.persistent_playback_feedback().map_or_else(
                        || self.playback_detail_line(playback),
                        |message| {
                            truncate_with_ellipsis(
                                &format!("              ERROR • {message}"),
                                usize::from(self.viewport.width),
                            )
                        },
                    ),
                    base,
                );
            }
            (None, None) => {
                buffer.set_string(0, top, " NOW PLAYING  Nothing playing", base);
                buffer.set_string(
                    0,
                    top + 1,
                    self.persistent_playback_feedback().map_or_else(
                        || "              Open a choice to keep exploring.".to_owned(),
                        |message| {
                            truncate_with_ellipsis(
                                &format!("              ERROR • {message}"),
                                usize::from(self.viewport.width),
                            )
                        },
                    ),
                    base,
                );
            }
        }
    }

    fn persistent_playback_feedback(&self) -> Option<&str> {
        match &self.playback_feedback {
            Some(PlaybackFeedback::Persistent(message)) => Some(message),
            Some(PlaybackFeedback::Brief(_)) | None => None,
        }
    }

    fn pending_target_line(&self, pending: &PendingPlayback, prefix: &str) -> String {
        match &pending.request.target {
            PlaybackTarget::Station(station_id) => {
                let station = self
                    .catalog
                    .station(station_id)
                    .expect("playback request should reference a catalog Station");
                protected_identity_line(
                    usize::from(self.viewport.width),
                    prefix,
                    station.name(),
                    self.catalog.source_badge(station.source_id()),
                    "",
                )
            }
            PlaybackTarget::Track(track_id) => {
                let track = self
                    .catalog
                    .track(track_id)
                    .expect("playback request should reference a catalog Track");
                protected_identity_line(
                    usize::from(self.viewport.width),
                    prefix,
                    track.name(),
                    self.catalog.source_badge(track.source_id()),
                    "",
                )
            }
        }
    }

    fn playback_summary_line(&self, playback: &PlaybackSession) -> String {
        match playback {
            PlaybackSession::Station { station_id, state } => {
                let station = self
                    .catalog
                    .station(station_id)
                    .expect("playback session should reference a catalog Station");
                protected_identity_line(
                    usize::from(self.viewport.width),
                    " NOW PLAYING  ",
                    station.name(),
                    self.catalog.source_badge(station.source_id()),
                    &format!(
                        " • {}",
                        match state {
                            StationPlaybackState::Live => "LIVE",
                            StationPlaybackState::Paused => "LIVE • PAUSED",
                        }
                    ),
                )
            }
            PlaybackSession::Track {
                current_track_id,
                elapsed_seconds,
                state,
                ..
            } => {
                let track = self
                    .catalog
                    .track(current_track_id)
                    .expect("playback session should reference a catalog Track");
                protected_identity_line(
                    usize::from(self.viewport.width),
                    " NOW PLAYING  ",
                    track.name(),
                    self.catalog.source_badge(track.source_id()),
                    &format!(
                        " • {} • {}/{}",
                        match state {
                            TrackPlaybackState::Playing => "PLAYING",
                            TrackPlaybackState::Paused => "PAUSED",
                            TrackPlaybackState::Stopped => "STOPPED",
                        },
                        Self::format_duration(*elapsed_seconds),
                        Self::format_duration(track.duration_seconds())
                    ),
                )
            }
        }
    }

    fn playback_detail_line(&self, playback: &PlaybackSession) -> String {
        match playback {
            PlaybackSession::Station { .. } => {
                "              Continuous Station • Queue empty".to_owned()
            }
            PlaybackSession::Track { queue, .. } => {
                let suffix = if queue.len() == 1 { "" } else { "s" };
                format!("              Queue {} Track{suffix}", queue.len())
            }
        }
    }

    fn render_help(&self, buffer: &mut Buffer, base: Style) {
        let destination = match &self.current.destination {
            Destination::Home => "HOME",
            Destination::ListeningIntents => "MOOD & ACTIVITY",
            Destination::ListeningIntent(intent_id) => self
                .catalog
                .listening_intent(intent_id)
                .map_or("MOOD & ACTIVITY", |intent| intent.name()),
            Destination::Stations => "RADIO STATIONS",
            Destination::Playlists => "MY PLAYLISTS",
            Destination::Services => "BROWSE SERVICES",
            Destination::ServiceCatalog(service_id) => self
                .catalog
                .service(service_id)
                .map_or("BROWSE SERVICES", |service| service.name()),
            Destination::Search => "SEARCH EVERYTHING",
            Destination::StationDetails { station_id, .. } => self
                .catalog
                .station(station_id)
                .map_or("STATION", |station| station.name()),
            Destination::PlaylistDetails { playlist_id, .. } => self
                .catalog
                .playlist(playlist_id)
                .map_or("PLAYLIST", |playlist| playlist.name()),
            Destination::NowPlaying => "NOW PLAYING",
            Destination::TrackDetails { track_id, .. } => self
                .catalog
                .track(track_id)
                .map_or("TRACK", |track| track.name()),
        };
        self.render_location(buffer, &format!(" SONGDIAL / {destination} / HELP"), base);
        buffer.set_string(0, 2, "  COMPLETE KEY GUIDE", base);
        buffer.set_string(0, 4, "  ↑ / k    Move selection up", base);
        buffer.set_string(0, 5, "  ↓ / j    Move selection down", base);
        buffer.set_string(0, 6, "  Enter    Open without playing", base);
        buffer.set_string(0, 7, "  p        Start a new Playback session", base);
        buffer.set_string(0, 8, "  Space    Pause, resume, or restart", base);
        buffer.set_string(
            0,
            9,
            "  a        Add a Track or Playlist to the Queue",
            base,
        );
        buffer.set_string(0, 10, "  d        Remove the selected queued Track", base);
        buffer.set_string(0, 11, "  /        Open Search or focus its query", base);
        buffer.set_string(0, 12, "  n        Open Now Playing and Queue", base);
        buffer.set_string(0, 13, "  Esc      Go back or close help", base);
        buffer.set_string(0, 14, "  ?        Show contextual help", base);
        buffer.set_string(0, 15, "  q / Ctrl+C  Quit", base);
        let local_help = match &self.current.destination {
            Destination::Home => "  Home: choose a listening path, then press Enter.",
            Destination::ListeningIntents => "  Mood & activity: choose what fits with Enter.",
            Destination::ListeningIntent(_) => "  Open a Station or Playlist with Enter.",
            Destination::Stations => "  Radio stations: inspect with Enter or play with p.",
            Destination::Playlists => "  My playlists: inspect with Enter or play with p.",
            Destination::Services => "  Browse services: choose a fictional Service with Enter.",
            Destination::ServiceCatalog(_) => {
                "  Service catalog: inspect shared Stations, Playlists, and Tracks."
            }
            Destination::Search => "  Search everything: type a query, then inspect a result.",
            Destination::StationDetails { .. } => {
                "  Station details: Esc returns to the exact prior selection."
            }
            Destination::PlaylistDetails { .. } => {
                "  Playlist details: browse Tracks; Esc restores the prior snapshot."
            }
            Destination::NowPlaying => {
                "  Now Playing: inspect, play, or remove the selected queued Track."
            }
            Destination::TrackDetails {
                origin: TrackDetailsOrigin::Queue,
                ..
            } => "  Queue Track details: Esc restores the exact Queue selection.",
            Destination::TrackDetails {
                origin: TrackDetailsOrigin::ServiceCatalog(_),
                ..
            } => "  Service Track details: Esc restores the exact catalog selection.",
            Destination::TrackDetails {
                origin: TrackDetailsOrigin::Search,
                ..
            } => "  Search Track details: Esc restores the exact Search result.",
        };
        buffer.set_string(0, 16, local_help, base);
        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " Esc close  ? close  q quit", base);
    }

    const fn now_playing_top(&self) -> u16 {
        self.viewport.height - 4
    }

    const fn guide_top(&self) -> u16 {
        self.viewport.height - 2
    }

    fn queue_visible_items(&self) -> usize {
        usize::from(self.viewport.height.saturating_sub(10))
    }

    fn dense_visible_items(&self) -> usize {
        usize::from(self.now_playing_top().saturating_sub(6) / 2)
    }

    fn playlist_visible_items(&self) -> usize {
        usize::from(self.now_playing_top().saturating_sub(7))
    }

    fn search_visible_items(&self) -> usize {
        usize::from(self.now_playing_top().saturating_sub(11))
    }

    fn selected_home_choice(&self) -> HomeChoice {
        HomeChoice::ALL[self.current.selection]
    }

    fn format_duration(seconds: u16) -> String {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }

    fn service_catalog_breadcrumb(&self, service_id: &CatalogId) -> String {
        format!(
            "BROWSE SERVICES / {}",
            self.catalog.source_name(service_id).to_uppercase()
        )
    }

    const fn dense_row_state(
        selected: bool,
        playing: bool,
        availability: &Availability,
    ) -> &'static str {
        match (selected, playing, availability) {
            (true, _, Availability::Unavailable(_)) => "SEL+UNAV >",
            (false, _, Availability::Unavailable(_)) => "UNAVAIL !",
            (true, true, _) => "SEL+PLAY >",
            (false, true, _) => "PLAYING *",
            (true, false, _) => "SELECTED >",
            (false, false, Availability::Loading) => "LOADING ~",
            (false, false, Availability::Available) => "",
        }
    }

    const fn availability_label(availability: &Availability) -> &'static str {
        match availability {
            Availability::Available => "AVAILABLE",
            Availability::Loading => "LOADING",
            Availability::Unavailable(_) => "UNAVAIL",
        }
    }

    fn track_row_status(track: &Track) -> String {
        match track.availability() {
            Availability::Available => Self::format_duration(track.duration_seconds()),
            Availability::Loading => "LOADING".to_owned(),
            Availability::Unavailable(_) => "UNAVAIL".to_owned(),
        }
    }

    fn render_dense_catalog_row(&self, buffer: &mut Buffer, base: Style, row: DenseCatalogRow<'_>) {
        let state = Self::dense_row_state(row.selected, row.playing, row.availability);
        let line = dense_catalog_line(
            self.browser_list_width(),
            state,
            row.kind,
            row.title,
            row.status,
            self.catalog.source_badge(row.source_id),
        );
        buffer.set_string(0, row.row, line, base);
        buffer.set_string(
            0,
            row.row + 1,
            truncate_with_ellipsis(
                &format!("             {} • {}", row.detail, row.status),
                usize::from(self.browser_list_width()),
            ),
            base,
        );
        if row.selected && self.current.active_pane == ActivePane::List {
            buffer.set_style(
                self.dense_list_selection_area(row.row, 2),
                self.selected_style(),
            );
        }
    }

    fn selected_style(&self) -> Style {
        if self.options.color_enabled {
            Style::default()
                .fg(Color::Rgb(27, 29, 28))
                .bg(Color::Rgb(214, 166, 75))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        }
    }

    fn loading_label(&self) -> &'static str {
        if !self.options.motion_enabled {
            return "Loading";
        }
        match self.motion_frame {
            0 => "Loading",
            1 => "Loading ·",
            2 => "Loading ··",
            _ => "Loading ···",
        }
    }

    fn dense_list_selection_area(&self, row: u16, height: u16) -> Rect {
        Rect::new(2, row, self.browser_list_width().saturating_sub(2), height)
    }

    fn browser_list_width(&self) -> u16 {
        self.wide_detail_separator_column()
            .unwrap_or(self.viewport.width)
    }
}

fn dense_catalog_line(
    width: u16,
    state: &str,
    kind: &str,
    title: &str,
    status: &str,
    source_badge: &str,
) -> String {
    const LEADING_WIDTH: usize = 2;
    const STATE_WIDTH: usize = 11;
    const KIND_WIDTH: usize = 10;
    const STATUS_WIDTH: usize = 10;
    const SOURCE_WIDTH: usize = 11;

    let width = usize::from(width);
    let fixed_width = LEADING_WIDTH + STATE_WIDTH + KIND_WIDTH + STATUS_WIDTH + SOURCE_WIDTH;
    let title_width = width.saturating_sub(fixed_width);
    let source = bracketed_source_badge(source_badge);

    [
        "  ".to_owned(),
        fit_left(state, STATE_WIDTH),
        fit_left(kind, KIND_WIDTH),
        fit_left(title, title_width),
        fit_right(status, STATUS_WIDTH),
        fit_right(&source, SOURCE_WIDTH),
    ]
    .concat()
}

fn protected_identity_line(
    width: usize,
    prefix: &str,
    title: &str,
    source_badge: &str,
    suffix: &str,
) -> String {
    let source = format!(" {}", bracketed_source_badge(source_badge));
    let fixed_width = UnicodeWidthStr::width(prefix)
        + UnicodeWidthStr::width(source.as_str())
        + UnicodeWidthStr::width(suffix);
    let title_width = width.saturating_sub(fixed_width);
    format!(
        "{prefix}{}{source}{suffix}",
        truncate_with_ellipsis(title, title_width)
    )
}

fn bracketed_source_badge(source_badge: &str) -> String {
    format!("[{}]", truncate_with_ellipsis(source_badge, 9))
}

fn fit_left(text: &str, width: usize) -> String {
    let clipped = truncate_with_ellipsis(text, width);
    let padding = width.saturating_sub(UnicodeWidthStr::width(clipped.as_str()));
    format!("{clipped}{}", " ".repeat(padding))
}

fn fit_right(text: &str, width: usize) -> String {
    let clipped = truncate_with_ellipsis(text, width);
    let padding = width.saturating_sub(UnicodeWidthStr::width(clipped.as_str()));
    format!("{}{clipped}", " ".repeat(padding))
}

fn sanitize_terminal_text(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn truncate_with_ellipsis(text: &str, width: usize) -> String {
    let sanitized = sanitize_terminal_text(text);
    let text = sanitized.as_str();
    if width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(text) <= width {
        return text.to_owned();
    }

    let content_width = width.saturating_sub(1);
    let mut clipped = String::new();
    let mut used_width = 0;
    for grapheme in UnicodeSegmentation::graphemes(text, true) {
        let grapheme_width = UnicodeWidthStr::width(grapheme);
        if grapheme_width == 0 && clipped.is_empty() {
            continue;
        }
        if used_width + grapheme_width > content_width {
            break;
        }
        clipped.push_str(grapheme);
        used_width += grapheme_width;
    }
    clipped.push('…');
    clipped
}

fn truncate_middle_with_ellipsis(text: &str, width: usize) -> String {
    let sanitized = sanitize_terminal_text(text.trim_end());
    if width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(sanitized.as_str()) <= width {
        return sanitized;
    }

    let content_width = width.saturating_sub(1);
    let leading_budget = content_width * 2 / 5;
    let mut leading = String::new();
    let mut leading_width = 0;
    for grapheme in UnicodeSegmentation::graphemes(sanitized.as_str(), true) {
        let grapheme_width = UnicodeWidthStr::width(grapheme);
        if leading_width + grapheme_width > leading_budget {
            break;
        }
        leading.push_str(grapheme);
        leading_width += grapheme_width;
    }

    let trailing_budget = content_width.saturating_sub(leading_width);
    let mut trailing_graphemes = Vec::new();
    let mut trailing_width = 0;
    for grapheme in UnicodeSegmentation::graphemes(sanitized.as_str(), true).rev() {
        let grapheme_width = UnicodeWidthStr::width(grapheme);
        if trailing_width + grapheme_width > trailing_budget {
            break;
        }
        trailing_graphemes.push(grapheme);
        trailing_width += grapheme_width;
    }
    trailing_graphemes.reverse();

    format!("{leading}…{}", trailing_graphemes.concat())
}

impl Viewport {
    const fn is_supported(self) -> bool {
        self.width >= 80 && self.height >= 24
    }
}
