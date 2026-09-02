use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::{
    Availability, CatalogId, DemoCatalog,
    catalog::{IntentMatch, Playlist, Station, Track},
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
                title: "MOOD & ACTIVITY",
            },
            Self::Stations => HomeChoiceText {
                label: "Radio stations",
                title: "RADIO STATIONS",
            },
            Self::Playlists => HomeChoiceText {
                label: "My playlists",
                title: "MY PLAYLISTS",
            },
            Self::Services => HomeChoiceText {
                label: "Browse services",
                title: "BROWSE SERVICES",
            },
            Self::Search => HomeChoiceText {
                label: "Search everything",
                title: "SEARCH EVERYTHING",
            },
        }
    }
}

struct HomeChoiceText {
    label: &'static str,
    title: &'static str,
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
pub enum Key {
    Up,
    Down,
    Enter,
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
    StationDetails {
        intent_id: CatalogId,
        station_id: CatalogId,
    },
    PlaylistDetails {
        intent_id: CatalogId,
        playlist_id: CatalogId,
    },
    NowPlaying,
    QueuedTrackDetails {
        track_id: CatalogId,
    },
    NotYetAvailable(HomeChoice),
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
}

impl DestinationSnapshot {
    const fn new(destination: Destination) -> Self {
        let active_pane = match &destination {
            Destination::StationDetails { .. }
            | Destination::QueuedTrackDetails { .. }
            | Destination::NotYetAvailable(_) => ActivePane::Details,
            Destination::Home
            | Destination::ListeningIntents
            | Destination::ListeningIntent(_)
            | Destination::PlaylistDetails { .. }
            | Destination::NowPlaying => ActivePane::List,
        };
        Self {
            destination,
            selection: 0,
            scroll_offset: 0,
            active_pane,
        }
    }
}

pub struct Application {
    viewport: Viewport,
    catalog: DemoCatalog,
    current: DestinationSnapshot,
    history: Vec<DestinationSnapshot>,
    help_visible: bool,
    playback: Option<PlaybackSession>,
    pending_playback: Option<PendingPlayback>,
    playback_feedback: Option<PlaybackFeedback>,
    next_playback_request_id: u64,
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
        Self::with_catalog(viewport, DemoCatalog::fixed())
    }

    #[must_use]
    pub fn with_catalog(viewport: Viewport, catalog: DemoCatalog) -> Self {
        Self {
            viewport,
            catalog,
            current: DestinationSnapshot::new(Destination::Home),
            history: Vec::new(),
            help_visible: false,
            playback: None,
            pending_playback: None,
            playback_feedback: None,
            next_playback_request_id: 1,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Effect {
        let Event::Key(key) = event else {
            return match event {
                Event::Resize(viewport) => {
                    self.viewport = viewport;
                    Effect::None
                }
                Event::Tick => {
                    if matches!(
                        self.playback_feedback.as_ref(),
                        Some(PlaybackFeedback::Brief(_))
                    ) {
                        self.playback_feedback = None;
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
            };
        };

        if matches!(key, Key::Char('q') | Key::CtrlC) {
            return Effect::Quit;
        }

        if !self.viewport.is_supported() {
            return Effect::None;
        }

        if key == Key::Char('?') {
            self.help_visible = !self.help_visible;
            return Effect::None;
        }

        if self.help_visible && key == Key::Escape {
            self.help_visible = false;
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
            Key::Char(' ') => self.toggle_playback(),
            Key::Char(_) | Key::CtrlC => {}
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
        let (track_ids, confirmation) = match &self.current.destination {
            Destination::PlaylistDetails { playlist_id, .. } => {
                let Some(track) = self
                    .catalog
                    .playlist(playlist_id)
                    .and_then(|playlist| playlist.track_ids().get(self.current.selection))
                    .and_then(|track_id| self.catalog.track(track_id))
                else {
                    return;
                };
                match track.availability() {
                    Availability::Available => {}
                    Availability::Loading => {
                        self.playback_feedback = Some(PlaybackFeedback::Persistent(format!(
                            "Cannot add: {} is still loading.",
                            track.name()
                        )));
                        return;
                    }
                    Availability::Unavailable(reason) => {
                        self.playback_feedback = Some(PlaybackFeedback::Persistent(format!(
                            "Cannot add: {reason}"
                        )));
                        return;
                    }
                }
                (
                    vec![track.id().clone()],
                    format!("Added {} to Queue.", track.name()),
                )
            }
            Destination::ListeningIntent(intent_id) => {
                let intent_matches = self.catalog.intent_matches(intent_id);
                let Some(intent_match) = intent_matches.get(self.current.selection) else {
                    return;
                };
                let IntentMatch::Playlist(playlist) = intent_match else {
                    self.playback_feedback = Some(PlaybackFeedback::Persistent(
                        "Cannot add: Stations are continuous and cannot be queued.".to_owned(),
                    ));
                    return;
                };
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
                    self.playback_feedback = Some(PlaybackFeedback::Persistent(format!(
                        "Cannot add: {} has no playable Tracks.",
                        playlist.name()
                    )));
                    return;
                }
                let suffix = if added_count == 1 { "" } else { "s" };
                (
                    track_ids,
                    format!(
                        "Added {} Track{suffix} from {}.",
                        added_count,
                        playlist.name()
                    ),
                )
            }
            Destination::Home
            | Destination::ListeningIntents
            | Destination::NowPlaying
            | Destination::QueuedTrackDetails { .. }
            | Destination::NotYetAvailable(_) => return,
            Destination::StationDetails { .. } => {
                self.playback_feedback = Some(PlaybackFeedback::Persistent(
                    "Cannot add: Stations are continuous and cannot be queued.".to_owned(),
                ));
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
            Destination::StationDetails { station_id, .. } => self
                .catalog
                .station(station_id)
                .map_or(PlaybackStartOutcome::NoPlayableSelected, |station| {
                    Self::station_playback_start(station)
                }),
            Destination::PlaylistDetails { playlist_id, .. } => self
                .catalog
                .playlist(playlist_id)
                .and_then(|playlist| playlist.track_ids().get(self.current.selection))
                .and_then(|track_id| self.catalog.track(track_id))
                .map_or(PlaybackStartOutcome::NoPlayableSelected, |track| {
                    Self::track_playback_start(track)
                }),
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
            Destination::Home
            | Destination::ListeningIntents
            | Destination::QueuedTrackDetails { .. }
            | Destination::NotYetAvailable(_) => PlaybackStartOutcome::NoPlayableSelected,
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
            Destination::ListeningIntent(_) => 7,
            Destination::PlaylistDetails { .. } => 13,
            Destination::NowPlaying => self.queue_visible_items(),
            Destination::Home => HomeChoice::ALL.len(),
            Destination::ListeningIntents => self.catalog.listening_intents().len(),
            Destination::StationDetails { .. }
            | Destination::QueuedTrackDetails { .. }
            | Destination::NotYetAvailable(_) => 0,
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
            Destination::ListeningIntent(intent_id) => self.catalog.intent_matches(intent_id).len(),
            Destination::PlaylistDetails { playlist_id, .. } => self
                .catalog
                .playlist(playlist_id)
                .map_or(0, |playlist| playlist.track_ids().len()),
            Destination::NowPlaying => match &self.playback {
                Some(PlaybackSession::Track { queue, .. }) => queue.len(),
                Some(PlaybackSession::Station { .. }) | None => 0,
            },
            Destination::StationDetails { .. }
            | Destination::QueuedTrackDetails { .. }
            | Destination::NotYetAvailable(_) => 0,
        }
    }

    fn open_selected(&mut self) {
        let destination = match self.current.destination.clone() {
            Destination::Home => {
                let choice = self.selected_home_choice();
                if choice == HomeChoice::ListeningIntents {
                    Destination::ListeningIntents
                } else {
                    Destination::NotYetAvailable(choice)
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
                        intent_id,
                        station_id: station.id().clone(),
                    },
                    Some(IntentMatch::Playlist(playlist)) => Destination::PlaylistDetails {
                        intent_id,
                        playlist_id: playlist.id().clone(),
                    },
                    None => return,
                }
            }
            Destination::NowPlaying => {
                let Some(PlaybackSession::Track { queue, .. }) = &self.playback else {
                    return;
                };
                let Some(track_id) = queue.get(self.current.selection) else {
                    return;
                };
                Destination::QueuedTrackDetails {
                    track_id: track_id.clone(),
                }
            }
            Destination::StationDetails { .. }
            | Destination::PlaylistDetails { .. }
            | Destination::QueuedTrackDetails { .. }
            | Destination::NotYetAvailable(_) => return,
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

    fn restore_previous_destination(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.current = previous;
        }
    }

    #[must_use]
    pub fn render(&self) -> Buffer {
        let area = Rect::new(0, 0, self.viewport.width, self.viewport.height);
        let mut buffer = Buffer::empty(area);
        let base = Style::default()
            .fg(Color::Rgb(222, 216, 202))
            .bg(Color::Rgb(27, 29, 28));
        buffer.set_style(area, base);

        if !self.viewport.is_supported() {
            return buffer;
        }

        if self.help_visible {
            self.render_help(&mut buffer, base);
        } else {
            self.render_destination(&mut buffer, base);
        }
        self.render_brief_feedback(&mut buffer, base);

        buffer
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
            Destination::StationDetails {
                intent_id,
                station_id,
            } => self.render_station_details(buffer, base, intent_id, station_id),
            Destination::PlaylistDetails {
                intent_id,
                playlist_id,
            } => self.render_playlist_details(buffer, base, intent_id, playlist_id),
            Destination::NowPlaying => self.render_now_playing_destination(buffer, base),
            Destination::QueuedTrackDetails { track_id } => {
                self.render_queued_track_details(buffer, base, track_id);
            }
            Destination::NotYetAvailable(choice) => {
                let title = choice.text().title;
                buffer.set_string(0, 0, format!(" SONGDIAL / {title}"), base);
                buffer.set_string(0, 2, format!("  {title}"), base);
                buffer.set_string(0, 4, "  This Destination is not yet available.", base);
                buffer.set_string(
                    0,
                    5,
                    "  Return Home to choose another listening path.",
                    base,
                );
                self.render_now_playing(buffer, base);
                buffer.set_string(
                    0,
                    self.guide_top(),
                    " n queue  Esc back  ? help  q quit",
                    base,
                );
            }
        }
    }

    fn render_home(&self, buffer: &mut Buffer, base: Style) {
        buffer.set_string(0, 0, " SONGDIAL / HOME", base);
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

        let selected = Self::selected_style();
        if self.current.active_pane == ActivePane::List {
            buffer.set_style(
                Rect::new(2, 4 + self.current.selection as u16, 76, 1),
                selected,
            );
        }
    }

    fn render_listening_intents(&self, buffer: &mut Buffer, base: Style) {
        buffer.set_string(0, 0, " SONGDIAL / MOOD & ACTIVITY", base);
        buffer.set_string(0, 2, "  MOOD & ACTIVITY", base);
        buffer.set_string(0, 3, "  Choose what fits right now.", base);

        for (index, intent) in self.catalog.listening_intents().iter().enumerate() {
            let line = if index == self.current.selection {
                format!("  > {:<58}SELECTED", intent.name())
            } else {
                format!("    {}", intent.name())
            };
            buffer.set_string(0, 5 + index as u16, line, base);
        }

        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " ↑/k up  ↓/j down  Enter open", base);
        buffer.set_string(0, self.guide_top() + 1, " n queue  Esc back", base);

        let selected = Self::selected_style();
        if self.current.active_pane == ActivePane::List {
            buffer.set_style(
                Rect::new(2, 5 + self.current.selection as u16, 76, 1),
                selected,
            );
        }
    }

    fn render_listening_intent(&self, buffer: &mut Buffer, base: Style, intent_id: &CatalogId) {
        let Some(intent) = self.catalog.listening_intent(intent_id) else {
            return;
        };
        let matches = self.catalog.intent_matches(intent_id);

        buffer.set_string(
            0,
            0,
            format!(
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
            buffer.set_string(0, 4, counts, base);
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
            .take(7)
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
                    matches!(
                        &self.playback,
                        Some(PlaybackSession::Track {
                            origin_playlist_id: Some(playlist_id),
                            ..
                        }) if playlist_id == playlist.id()
                    ),
                ),
            };
            let state = Self::dense_row_state(selected, playing, availability);
            let source = format!("[{}]", self.catalog.source_badge(source_id));
            let availability = match availability {
                Availability::Available => "AVAILABLE",
                Availability::Loading => "LOADING",
                Availability::Unavailable(_) => "UNAVAIL",
            };

            buffer.set_string(
                0,
                row,
                format!("  {state:<11}{kind:<10}{title:<46}{source:>11}"),
                base,
            );
            buffer.set_string(
                0,
                row + 1,
                format!("             {detail} • {availability}"),
                base,
            );

            if selected && self.current.active_pane == ActivePane::List {
                let selected = Self::selected_style();
                buffer.set_style(Self::dense_list_selection_area(row, 2), selected);
            }
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

    fn render_station_details(
        &self,
        buffer: &mut Buffer,
        base: Style,
        intent_id: &CatalogId,
        station_id: &CatalogId,
    ) {
        let intent = self
            .catalog
            .listening_intent(intent_id)
            .expect("fixed Station parent intent should exist");
        let station = self
            .catalog
            .station(station_id)
            .expect("fixed Station should exist");
        let status = match station.availability() {
            Availability::Available => "Available",
            Availability::Loading => "Loading",
            Availability::Unavailable(_) => "Unavailable",
        };

        buffer.set_string(
            0,
            0,
            format!(
                " SONGDIAL / MOOD & ACTIVITY / {} / {}",
                intent.name().to_uppercase(),
                station.name().to_uppercase()
            ),
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
        buffer.set_string(0, self.guide_top(), " p play  Space pause  n queue", base);
        buffer.set_string(0, self.guide_top() + 1, " Esc back  ? help", base);
    }

    fn render_playlist_details(
        &self,
        buffer: &mut Buffer,
        base: Style,
        intent_id: &CatalogId,
        playlist_id: &CatalogId,
    ) {
        let intent = self
            .catalog
            .listening_intent(intent_id)
            .expect("fixed Playlist parent intent should exist");
        let playlist = self
            .catalog
            .playlist(playlist_id)
            .expect("fixed Playlist should exist");
        let tracks = self.catalog.playlist_tracks(playlist_id);

        buffer.set_string(
            0,
            0,
            format!(
                " SONGDIAL / MOOD & ACTIVITY / {} / {}",
                intent.name().to_uppercase(),
                playlist.name().to_uppercase()
            ),
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
            buffer.set_string(0, 6, "  TRACKS • Empty", base);
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
                    intent.name()
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
            .take(13)
            .enumerate()
        {
            let index = self.current.scroll_offset + slot;
            let selected = index == self.current.selection;
            let playing = matches!(
                &self.playback,
                Some(PlaybackSession::Track {
                    current_track_id,
                    ..
                }) if current_track_id == track.id()
            );
            let state = Self::dense_row_state(selected, playing, track.availability());
            let source = format!("[{}]", self.catalog.source_badge(track.source_id()));
            let row = 7 + slot as u16;

            buffer.set_string(
                0,
                row,
                format!(
                    "  {state:<11}{:<8}{:<28}{:<20}{source:>11}",
                    "TRACK",
                    track.name(),
                    track.creator()
                ),
                base,
            );

            if selected && self.current.active_pane == ActivePane::List {
                buffer.set_style(
                    Self::dense_list_selection_area(row, 1),
                    Self::selected_style(),
                );
            }
        }

        self.render_now_playing(buffer, base);
        if playlist.track_ids().is_empty() {
            buffer.set_string(0, self.guide_top(), " Esc back  ? help  q quit", base);
        } else {
            buffer.set_string(0, self.guide_top(), " ↑/k ↓/j move  p play", base);
            buffer.set_string(0, self.guide_top() + 1, " a add  n queue  Esc back", base);
        }
    }

    fn render_now_playing_destination(&self, buffer: &mut Buffer, base: Style) {
        buffer.set_string(0, 0, " SONGDIAL / NOW PLAYING", base);
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
                let source = format!("[{}]", self.catalog.source_badge(track.source_id()));
                let row = 6 + slot as u16;
                buffer.set_string(
                    0,
                    row,
                    format!(
                        "  {state:<11}{:<8}{:<28}{:<20}{source:>11}",
                        "TRACK",
                        track.name(),
                        track.creator()
                    ),
                    base,
                );
                if selected && self.current.active_pane == ActivePane::List {
                    buffer.set_style(
                        Self::dense_list_selection_area(row, 1),
                        Self::selected_style(),
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

    fn render_queued_track_details(&self, buffer: &mut Buffer, base: Style, track_id: &CatalogId) {
        let track = self
            .catalog
            .track(track_id)
            .expect("Queue details should reference a catalog Track");
        let status = match track.availability() {
            Availability::Available => "Available",
            Availability::Loading => "Loading",
            Availability::Unavailable(_) => "Unavailable",
        };
        buffer.set_string(
            0,
            0,
            format!(" SONGDIAL / NOW PLAYING / {}", track.name().to_uppercase()),
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
        self.render_now_playing(buffer, base);
        buffer.set_string(
            0,
            self.guide_top(),
            " Space pause  Esc back  ? help  q quit",
            base,
        );
    }

    fn render_now_playing(&self, buffer: &mut Buffer, base: Style) {
        let top = self.now_playing_top();
        match (&self.playback, &self.pending_playback) {
            (Some(playback), Some(pending)) => {
                buffer.set_string(0, top, self.playback_summary_line(playback), base);
                buffer.set_string(
                    0,
                    top + 1,
                    format!(
                        "              Loading {}",
                        self.pending_target_label(pending)
                    ),
                    base,
                );
            }
            (None, Some(pending)) => {
                buffer.set_string(
                    0,
                    top,
                    format!(
                        " NOW PLAYING  Loading {}",
                        self.pending_target_label(pending)
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
                        |message| format!("              {message}"),
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
                        |message| format!("              {message}"),
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

    fn pending_target_label(&self, pending: &PendingPlayback) -> String {
        match &pending.request.target {
            PlaybackTarget::Station(station_id) => {
                let station = self
                    .catalog
                    .station(station_id)
                    .expect("playback request should reference a catalog Station");
                format!(
                    "{} [{}]",
                    station.name(),
                    self.catalog.source_badge(station.source_id())
                )
            }
            PlaybackTarget::Track(track_id) => {
                let track = self
                    .catalog
                    .track(track_id)
                    .expect("playback request should reference a catalog Track");
                format!(
                    "{} [{}]",
                    track.name(),
                    self.catalog.source_badge(track.source_id())
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
                format!(
                    " NOW PLAYING  {} [{}] • {}",
                    station.name(),
                    self.catalog.source_badge(station.source_id()),
                    match state {
                        StationPlaybackState::Live => "LIVE",
                        StationPlaybackState::Paused => "LIVE • PAUSED",
                    }
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
                format!(
                    " NOW PLAYING  {} [{}] • {} • {}/{}",
                    track.name(),
                    self.catalog.source_badge(track.source_id()),
                    match state {
                        TrackPlaybackState::Playing => "PLAYING",
                        TrackPlaybackState::Paused => "PAUSED",
                        TrackPlaybackState::Stopped => "STOPPED",
                    },
                    Self::format_duration(*elapsed_seconds),
                    Self::format_duration(track.duration_seconds())
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
            Destination::StationDetails { station_id, .. } => self
                .catalog
                .station(station_id)
                .map_or("STATION", |station| station.name()),
            Destination::PlaylistDetails { playlist_id, .. } => self
                .catalog
                .playlist(playlist_id)
                .map_or("PLAYLIST", |playlist| playlist.name()),
            Destination::NowPlaying => "NOW PLAYING",
            Destination::QueuedTrackDetails { track_id } => self
                .catalog
                .track(track_id)
                .map_or("TRACK", |track| track.name()),
            Destination::NotYetAvailable(choice) => choice.text().title,
        };
        buffer.set_string(0, 0, format!(" SONGDIAL / {destination} / HELP"), base);
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
        buffer.set_string(0, 11, "  n        Open Now Playing and Queue", base);
        buffer.set_string(0, 12, "  Esc      Go back or close help", base);
        buffer.set_string(0, 13, "  ?        Show contextual help", base);
        buffer.set_string(0, 14, "  q / Ctrl+C  Quit", base);
        let local_help = match &self.current.destination {
            Destination::Home => "  Home: choose a listening path, then press Enter.",
            Destination::ListeningIntents => "  Mood & activity: choose what fits with Enter.",
            Destination::ListeningIntent(_) => "  Open a Station or Playlist with Enter.",
            Destination::StationDetails { .. } => {
                "  Station details: Esc returns to the exact prior selection."
            }
            Destination::PlaylistDetails { .. } => {
                "  Playlist details: browse Tracks; Esc restores the prior snapshot."
            }
            Destination::NowPlaying => {
                "  Now Playing: inspect, play, or remove the selected queued Track."
            }
            Destination::QueuedTrackDetails { .. } => {
                "  Queue Track details: Esc restores the exact Queue selection."
            }
            Destination::NotYetAvailable(_) => "  This Destination has no additional actions yet.",
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

    fn selected_home_choice(&self) -> HomeChoice {
        HomeChoice::ALL[self.current.selection]
    }

    fn format_duration(seconds: u16) -> String {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
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

    fn selected_style() -> Style {
        Style::default()
            .fg(Color::Rgb(27, 29, 28))
            .bg(Color::Rgb(214, 166, 75))
            .add_modifier(Modifier::BOLD)
    }

    const fn dense_list_selection_area(row: u16, height: u16) -> Rect {
        Rect::new(2, row, 78, height)
    }
}

impl Viewport {
    const fn is_supported(self) -> bool {
        self.width >= 80 && self.height >= 24
    }
}
