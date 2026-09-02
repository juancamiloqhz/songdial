use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::{Availability, CatalogId, DemoCatalog, catalog::IntentMatch};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    Key(Key),
    Tick,
    Resize(Viewport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    None,
    Quit,
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
            Destination::StationDetails { .. } | Destination::NotYetAvailable(_) => {
                ActivePane::Details
            }
            Destination::Home
            | Destination::ListeningIntents
            | Destination::ListeningIntent(_)
            | Destination::PlaylistDetails { .. } => ActivePane::List,
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
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Effect {
        if let Event::Resize(viewport) = event {
            self.viewport = viewport;
            return Effect::None;
        }

        if event == Event::Tick {
            return Effect::None;
        }

        if matches!(event, Event::Key(Key::Char('q') | Key::CtrlC)) {
            return Effect::Quit;
        }

        if !self.viewport.is_supported() {
            return Effect::None;
        }

        if event == Event::Key(Key::Char('?')) {
            self.help_visible = !self.help_visible;
            return Effect::None;
        }

        if self.help_visible && event == Event::Key(Key::Escape) {
            self.help_visible = false;
            return Effect::None;
        }

        match event {
            Event::Key(Key::Down | Key::Char('j')) => self.move_selection_down(),
            Event::Key(Key::Up | Key::Char('k')) => self.move_selection_up(),
            Event::Key(Key::Enter) => self.open_selected(),
            Event::Key(Key::Escape) => self.restore_previous_destination(),
            Event::Key(_) => {}
            Event::Tick => unreachable!("tick events return before navigation"),
            Event::Resize(_) => unreachable!("resize events return before navigation"),
        }

        Effect::None
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
            Destination::Home => HomeChoice::ALL.len(),
            Destination::ListeningIntents => self.catalog.listening_intents().len(),
            Destination::StationDetails { .. } | Destination::NotYetAvailable(_) => 0,
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
            Destination::StationDetails { .. } | Destination::NotYetAvailable(_) => 0,
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
            Destination::StationDetails { .. }
            | Destination::PlaylistDetails { .. }
            | Destination::NotYetAvailable(_) => return,
        };

        self.history.push(self.current.clone());
        self.current = DestinationSnapshot::new(destination);
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

        buffer
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
                buffer.set_string(0, self.guide_top(), " Esc back  ? help  q quit", base);
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
        buffer.set_string(0, self.guide_top() + 1, " ? help  q quit", base);

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
        buffer.set_string(0, self.guide_top() + 1, " Esc back  ? help  q quit", base);

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
            let state = if index == self.current.selection {
                "SELECTED >"
            } else {
                ""
            };
            let (kind, title, source_id, detail, availability) = match intent_match {
                IntentMatch::Station(station) => (
                    "STATION",
                    station.name(),
                    station.source_id(),
                    station.style(),
                    station.availability(),
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
                ),
            };
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

            if index == self.current.selection && self.current.active_pane == ActivePane::List {
                let selected = Self::selected_style();
                buffer.set_style(Self::dense_list_selection_area(row, 2), selected);
            }
        }

        self.render_now_playing(buffer, base);
        if matches.is_empty() {
            buffer.set_string(0, self.guide_top(), " Esc back  ? help  q quit", base);
        } else {
            buffer.set_string(
                0,
                self.guide_top(),
                " ↑/k up  ↓/j down  Enter inspect",
                base,
            );
            buffer.set_string(0, self.guide_top() + 1, " Esc back  ? help  q quit", base);
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
        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " Esc back  ? help  q quit", base);
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
            let state = match (selected, track.availability()) {
                (true, Availability::Unavailable(_)) => "SEL+UNAV >",
                (true, _) => "SELECTED >",
                (false, Availability::Unavailable(_)) => "UNAVAIL !",
                (false, Availability::Loading) => "LOADING ~",
                (false, Availability::Available) => "",
            };
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
            buffer.set_string(0, self.guide_top(), " ↑/k up  ↓/j down  Esc back", base);
            buffer.set_string(0, self.guide_top() + 1, " ? help  q quit", base);
        }
    }

    fn render_now_playing(&self, buffer: &mut Buffer, base: Style) {
        let top = self.now_playing_top();
        buffer.set_string(0, top, " NOW PLAYING  Nothing playing", base);
        buffer.set_string(
            0,
            top + 1,
            "              Open a choice to keep exploring.",
            base,
        );
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
            Destination::NotYetAvailable(choice) => choice.text().title,
        };
        buffer.set_string(0, 0, format!(" SONGDIAL / {destination} / HELP"), base);
        buffer.set_string(0, 2, "  COMPLETE KEY GUIDE", base);
        buffer.set_string(0, 4, "  ↑ / k    Move selection up", base);
        buffer.set_string(0, 5, "  ↓ / j    Move selection down", base);
        buffer.set_string(0, 6, "  Enter    Open without playing", base);
        buffer.set_string(0, 7, "  Esc      Go back or close help", base);
        buffer.set_string(0, 8, "  ?        Show contextual help", base);
        buffer.set_string(0, 9, "  q        Quit", base);
        buffer.set_string(0, 10, "  Ctrl+C   Quit immediately", base);
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
            Destination::NotYetAvailable(_) => "  This Destination has no additional actions yet.",
        };
        buffer.set_string(0, 12, local_help, base);
        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " Esc close  ? close  q quit", base);
    }

    const fn now_playing_top(&self) -> u16 {
        self.viewport.height - 4
    }

    const fn guide_top(&self) -> u16 {
        self.viewport.height - 2
    }

    fn selected_home_choice(&self) -> HomeChoice {
        HomeChoice::ALL[self.current.selection]
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
