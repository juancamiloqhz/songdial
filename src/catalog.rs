#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CatalogId(String);

impl CatalogId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Availability {
    Available,
    Loading,
    Unavailable(String),
}

impl Availability {
    #[must_use]
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable(reason.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CatalogIdentity {
    id: CatalogId,
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Service {
    identity: CatalogIdentity,
    badge: String,
}

impl Service {
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, badge: impl Into<String>) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId::new(id),
                name: name.into(),
            },
            badge: badge.into(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &CatalogId {
        &self.identity.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    #[must_use]
    pub fn badge(&self) -> &str {
        &self.badge
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListeningIntent {
    identity: CatalogIdentity,
    description: String,
    station_ids: Vec<CatalogId>,
    playlist_ids: Vec<CatalogId>,
}

impl ListeningIntent {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        station_ids: &[&str],
        playlist_ids: &[&str],
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId::new(id),
                name: name.into(),
            },
            description: description.into(),
            station_ids: station_ids.iter().map(|id| CatalogId::new(*id)).collect(),
            playlist_ids: playlist_ids.iter().map(|id| CatalogId::new(*id)).collect(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &CatalogId {
        &self.identity.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn station_ids(&self) -> &[CatalogId] {
        &self.station_ids
    }

    #[must_use]
    pub fn playlist_ids(&self) -> &[CatalogId] {
        &self.playlist_ids
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Station {
    identity: CatalogIdentity,
    style: String,
    description: String,
    source_id: CatalogId,
    availability: Availability,
}

impl Station {
    #[must_use]
    pub fn available(
        id: impl Into<String>,
        name: impl Into<String>,
        style: impl Into<String>,
        description: impl Into<String>,
        source_id: impl Into<String>,
    ) -> Self {
        Self::with_availability(
            id,
            name,
            style,
            description,
            source_id,
            Availability::Available,
        )
    }

    #[must_use]
    pub fn loading(
        id: impl Into<String>,
        name: impl Into<String>,
        style: impl Into<String>,
        description: impl Into<String>,
        source_id: impl Into<String>,
    ) -> Self {
        Self::with_availability(
            id,
            name,
            style,
            description,
            source_id,
            Availability::Loading,
        )
    }

    #[must_use]
    pub fn unavailable(
        id: impl Into<String>,
        name: impl Into<String>,
        style: impl Into<String>,
        description: impl Into<String>,
        source_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::with_availability(
            id,
            name,
            style,
            description,
            source_id,
            Availability::unavailable(reason),
        )
    }

    fn with_availability(
        id: impl Into<String>,
        name: impl Into<String>,
        style: impl Into<String>,
        description: impl Into<String>,
        source_id: impl Into<String>,
        availability: Availability,
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId::new(id),
                name: name.into(),
            },
            style: style.into(),
            description: description.into(),
            source_id: CatalogId::new(source_id),
            availability,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &CatalogId {
        &self.identity.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    #[must_use]
    pub fn style(&self) -> &str {
        &self.style
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub const fn source_id(&self) -> &CatalogId {
        &self.source_id
    }

    #[must_use]
    pub const fn availability(&self) -> &Availability {
        &self.availability
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Playlist {
    identity: CatalogIdentity,
    description: String,
    source_id: CatalogId,
    availability: Availability,
    track_ids: Vec<CatalogId>,
}

impl Playlist {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        source_id: impl Into<String>,
        track_ids: &[&str],
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId::new(id),
                name: name.into(),
            },
            description: description.into(),
            source_id: CatalogId::new(source_id),
            availability: Availability::Available,
            track_ids: track_ids.iter().map(|id| CatalogId::new(*id)).collect(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &CatalogId {
        &self.identity.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub const fn source_id(&self) -> &CatalogId {
        &self.source_id
    }

    #[must_use]
    pub const fn availability(&self) -> &Availability {
        &self.availability
    }

    #[must_use]
    pub fn track_ids(&self) -> &[CatalogId] {
        &self.track_ids
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Track {
    identity: CatalogIdentity,
    creator: String,
    source_id: CatalogId,
    duration_seconds: u16,
    availability: Availability,
}

impl Track {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        creator: impl Into<String>,
        source_id: impl Into<String>,
        duration_seconds: u16,
    ) -> Self {
        Self::with_availability(
            id,
            name,
            creator,
            source_id,
            duration_seconds,
            Availability::Available,
        )
    }

    #[must_use]
    pub fn unavailable(
        id: impl Into<String>,
        name: impl Into<String>,
        creator: impl Into<String>,
        source_id: impl Into<String>,
        duration_seconds: u16,
        reason: impl Into<String>,
    ) -> Self {
        Self::with_availability(
            id,
            name,
            creator,
            source_id,
            duration_seconds,
            Availability::unavailable(reason),
        )
    }

    fn with_availability(
        id: impl Into<String>,
        name: impl Into<String>,
        creator: impl Into<String>,
        source_id: impl Into<String>,
        duration_seconds: u16,
        availability: Availability,
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId::new(id),
                name: name.into(),
            },
            creator: creator.into(),
            source_id: CatalogId::new(source_id),
            duration_seconds,
            availability,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &CatalogId {
        &self.identity.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    #[must_use]
    pub fn creator(&self) -> &str {
        &self.creator
    }

    #[must_use]
    pub const fn source_id(&self) -> &CatalogId {
        &self.source_id
    }

    #[must_use]
    pub const fn duration_seconds(&self) -> u16 {
        self.duration_seconds
    }

    #[must_use]
    pub const fn availability(&self) -> &Availability {
        &self.availability
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IntentMatch<'a> {
    Station(&'a Station),
    Playlist(&'a Playlist),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ServiceCatalogItem<'a> {
    Station(&'a Station),
    Playlist(&'a Playlist),
    Track(&'a Track),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SearchResultGroup {
    ListeningIntents,
    Stations,
    Playlists,
    Tracks,
}

impl SearchResultGroup {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::ListeningIntents => "LISTENING INTENTS",
            Self::Stations => "STATIONS",
            Self::Playlists => "PLAYLISTS",
            Self::Tracks => "TRACKS",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SearchResult<'a> {
    ListeningIntent(&'a ListeningIntent),
    Station(&'a Station),
    Playlist(&'a Playlist),
    Track(&'a Track),
}

impl SearchResult<'_> {
    pub(crate) const fn group(self) -> SearchResultGroup {
        match self {
            Self::ListeningIntent(_) => SearchResultGroup::ListeningIntents,
            Self::Station(_) => SearchResultGroup::Stations,
            Self::Playlist(_) => SearchResultGroup::Playlists,
            Self::Track(_) => SearchResultGroup::Tracks,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SearchResults<'a> {
    pub(crate) items: Vec<SearchResult<'a>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServiceCatalogCounts {
    pub(crate) stations: usize,
    pub(crate) playlists: usize,
    pub(crate) tracks: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct ServiceCatalogView<'a> {
    pub(crate) items: Vec<ServiceCatalogItem<'a>>,
    pub(crate) counts: ServiceCatalogCounts,
}

/// Catalog data for the first Songdial milestone.
///
/// [`Self::fixed`] returns the deterministic fictional fixture, while
/// [`Self::new`] lets the application receive replacement local data. This is
/// not a schema for future Service integrations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemoCatalog {
    services: Vec<Service>,
    listening_intents: Vec<ListeningIntent>,
    stations: Vec<Station>,
    playlists: Vec<Playlist>,
    tracks: Vec<Track>,
}

impl DemoCatalog {
    #[must_use]
    pub fn new(
        services: Vec<Service>,
        listening_intents: Vec<ListeningIntent>,
        stations: Vec<Station>,
        playlists: Vec<Playlist>,
        tracks: Vec<Track>,
    ) -> Self {
        Self {
            services,
            listening_intents,
            stations,
            playlists,
            tracks,
        }
    }

    #[must_use]
    pub fn fixed() -> Self {
        Self {
            services: vec![
                Service::new("morrow-audio", "Morrow Audio", "MORROW"),
                Service::new("harbor-sound", "Harbor Sound", "HARBOR"),
            ],
            listening_intents: vec![
                ListeningIntent::new(
                    "deep-work",
                    "Deep Work",
                    "Steady, low-distraction sound for sustained concentration.",
                    &[
                        "morrow-night-ledger",
                        "harbor-daylight-circuit",
                        "morrow-stillwater-fm",
                        "harbor-kinetic-line",
                        "morrow-low-tide-radio",
                        "harbor-afterglow-signal",
                        "morrow-northbound-static",
                        "harbor-open-frequency",
                    ],
                    &[
                        "morrow-deep-work-rotation",
                        "harbor-focus-lines",
                        "morrow-calm-current",
                        "harbor-energy-shift",
                        "harbor-empty-room",
                    ],
                ),
                ListeningIntent::new(
                    "focus",
                    "Focus",
                    "Clear patterns that keep attention anchored.",
                    &[
                        "morrow-night-ledger",
                        "harbor-daylight-circuit",
                        "morrow-stillwater-fm",
                    ],
                    &["morrow-deep-work-rotation", "harbor-focus-lines"],
                ),
                ListeningIntent::new(
                    "flow",
                    "Flow",
                    "Forward motion without sharp interruptions.",
                    &["morrow-night-ledger", "harbor-kinetic-line"],
                    &["morrow-deep-work-rotation", "harbor-energy-shift"],
                ),
                ListeningIntent::new(
                    "calm",
                    "Calm",
                    "Soft textures and patient pacing for quieter moments.",
                    &[
                        "morrow-stillwater-fm",
                        "morrow-low-tide-radio",
                        "harbor-afterglow-signal",
                    ],
                    &["morrow-calm-current", "harbor-empty-room"],
                ),
                ListeningIntent::new(
                    "energy",
                    "Energy",
                    "Bright rhythm and momentum for an active stretch.",
                    &[
                        "harbor-daylight-circuit",
                        "harbor-kinetic-line",
                        "harbor-open-frequency",
                    ],
                    &["harbor-energy-shift"],
                ),
                ListeningIntent::new(
                    "reset",
                    "Reset",
                    "A clean break before choosing what comes next.",
                    &[],
                    &[],
                ),
            ],
            stations: vec![
                Station::available(
                    "morrow-night-ledger",
                    "Night Ledger",
                    "Ambient",
                    "Unhurried ambient transmissions for sustained attention.",
                    "morrow-audio",
                ),
                Station::loading(
                    "harbor-daylight-circuit",
                    "Daylight Circuit",
                    "Minimal electronic",
                    "A bright pulse that stays behind the work.",
                    "harbor-sound",
                ),
                Station::available(
                    "morrow-stillwater-fm",
                    "Stillwater FM",
                    "Ambient piano",
                    "Sparse piano and room tone with long breathing space.",
                    "morrow-audio",
                ),
                Station::available(
                    "harbor-kinetic-line",
                    "Kinetic Line",
                    "Instrumental pulse",
                    "Measured rhythmic instrumentals with no vocals.",
                    "harbor-sound",
                ),
                Station::available(
                    "morrow-low-tide-radio",
                    "Low Tide Radio",
                    "Downtempo",
                    "Slow electronic currents for a settled pace.",
                    "morrow-audio",
                ),
                Station::available(
                    "harbor-afterglow-signal",
                    "Afterglow Signal",
                    "Warm electronica",
                    "Soft analog color for late-day concentration.",
                    "harbor-sound",
                ),
                Station::unavailable(
                    "morrow-northbound-static",
                    "Northbound Static",
                    "Drone",
                    "Long-form tonal broadcasts from the northern line.",
                    "morrow-audio",
                    "Signal maintenance is in progress.",
                ),
                Station::available(
                    "harbor-open-frequency",
                    "Open Frequency",
                    "Indie instrumental",
                    "Guitar-led instrumentals with a steady horizon.",
                    "harbor-sound",
                ),
            ],
            playlists: vec![
                Playlist::new(
                    "morrow-deep-work-rotation",
                    "Deep Work Rotation",
                    "Twenty patient Tracks arranged for a long concentration block.",
                    "morrow-audio",
                    &[
                        "morrow-night-geometry",
                        "harbor-night-geometry",
                        "morrow-stillwater-signal",
                        "harbor-stillwater-signal",
                        "morrow-slow-aperture",
                        "harbor-soft-machines",
                        "morrow-quiet-index",
                        "harbor-glass-hours",
                        "morrow-copper-rain",
                        "harbor-parallel-dawn",
                        "morrow-signal-garden",
                        "harbor-warm-circuit",
                        "morrow-northern-room",
                        "harbor-paper-satellites",
                        "morrow-low-light-method",
                        "harbor-blueprint-sky",
                        "morrow-silent-meter",
                        "harbor-night-bloom",
                        "morrow-drift-calculus",
                        "harbor-cinder-lines",
                    ],
                ),
                Playlist::new(
                    "harbor-focus-lines",
                    "Focus Lines",
                    "Clean electronic lines for shorter focused sessions.",
                    "harbor-sound",
                    &[
                        "morrow-night-geometry",
                        "harbor-night-geometry",
                        "morrow-slow-aperture",
                        "harbor-soft-machines",
                    ],
                ),
                Playlist::new(
                    "morrow-calm-current",
                    "Calm Current",
                    "Quiet recordings that make room around the Listener.",
                    "morrow-audio",
                    &[
                        "morrow-stillwater-signal",
                        "harbor-stillwater-signal",
                        "morrow-quiet-index",
                        "harbor-glass-hours",
                        "morrow-copper-rain",
                        "harbor-parallel-dawn",
                    ],
                ),
                Playlist::new(
                    "harbor-energy-shift",
                    "Energy Shift",
                    "A compact rise from warm pulse to bright momentum.",
                    "harbor-sound",
                    &[
                        "morrow-signal-garden",
                        "harbor-warm-circuit",
                        "morrow-archive-of-air",
                        "harbor-bright-current",
                        "harbor-return-path",
                    ],
                ),
                Playlist::new(
                    "harbor-empty-room",
                    "Empty Room",
                    "A saved Playlist waiting for its first Track.",
                    "harbor-sound",
                    &[],
                ),
            ],
            tracks: vec![
                Track::new(
                    "morrow-night-geometry",
                    "Night Geometry",
                    "Sable Circuit",
                    "morrow-audio",
                    328,
                ),
                Track::new(
                    "harbor-night-geometry",
                    "Night Geometry",
                    "Sable Circuit",
                    "harbor-sound",
                    331,
                ),
                Track::new(
                    "morrow-stillwater-signal",
                    "Stillwater Signal",
                    "Vale Static",
                    "morrow-audio",
                    284,
                ),
                Track::new(
                    "harbor-stillwater-signal",
                    "Stillwater Signal",
                    "Vale Static",
                    "harbor-sound",
                    286,
                ),
                Track::new(
                    "morrow-slow-aperture",
                    "Slow Aperture",
                    "Mira Form",
                    "morrow-audio",
                    302,
                ),
                Track::new(
                    "harbor-soft-machines",
                    "Soft Machines",
                    "Coast Index",
                    "harbor-sound",
                    276,
                ),
                Track::new(
                    "morrow-quiet-index",
                    "Quiet Index",
                    "Oren Vale",
                    "morrow-audio",
                    315,
                ),
                Track::new(
                    "harbor-glass-hours",
                    "Glass Hours",
                    "Lumen Archive",
                    "harbor-sound",
                    247,
                ),
                Track::new(
                    "morrow-copper-rain",
                    "Copper Rain",
                    "Iris Method",
                    "morrow-audio",
                    293,
                ),
                Track::new(
                    "harbor-parallel-dawn",
                    "Parallel Dawn",
                    "Drift Assembly",
                    "harbor-sound",
                    261,
                ),
                Track::new(
                    "morrow-signal-garden",
                    "Signal Garden",
                    "Theo Current",
                    "morrow-audio",
                    308,
                ),
                Track::new(
                    "harbor-warm-circuit",
                    "Warm Circuit",
                    "Harborline",
                    "harbor-sound",
                    255,
                ),
                Track::new(
                    "morrow-northern-room",
                    "Northern Room",
                    "Mara Field",
                    "morrow-audio",
                    322,
                ),
                Track::new(
                    "harbor-paper-satellites",
                    "Paper Satellites",
                    "Kite Bureau",
                    "harbor-sound",
                    269,
                ),
                Track::new(
                    "morrow-low-light-method",
                    "Low Light Method",
                    "Sable Circuit",
                    "morrow-audio",
                    297,
                ),
                Track::unavailable(
                    "harbor-blueprint-sky",
                    "Blueprint Sky",
                    "Grey Atlas",
                    "harbor-sound",
                    244,
                    "This recording is unavailable in the Demo catalog.",
                ),
                Track::new(
                    "morrow-silent-meter",
                    "Silent Meter",
                    "Vale Static",
                    "morrow-audio",
                    311,
                ),
                Track::new(
                    "harbor-night-bloom",
                    "Night Bloom",
                    "Nara Loom",
                    "harbor-sound",
                    278,
                ),
                Track::new(
                    "morrow-drift-calculus",
                    "Drift Calculus",
                    "Mono Pines",
                    "morrow-audio",
                    305,
                ),
                Track::new(
                    "harbor-cinder-lines",
                    "Cinder Lines",
                    "Ember Index",
                    "harbor-sound",
                    252,
                ),
                Track::new(
                    "morrow-archive-of-air",
                    "Archive of Air",
                    "Quiet Signal",
                    "morrow-audio",
                    319,
                ),
                Track::new(
                    "harbor-bright-current",
                    "Bright Current",
                    "Solar Bureau",
                    "harbor-sound",
                    263,
                ),
                Track::new(
                    "morrow-long-form",
                    "Long Form",
                    "Resting State",
                    "morrow-audio",
                    300,
                ),
                Track::new(
                    "harbor-return-path",
                    "Return Path",
                    "Coast Index",
                    "harbor-sound",
                    271,
                ),
            ],
        }
    }

    #[must_use]
    pub fn services(&self) -> &[Service] {
        &self.services
    }

    #[must_use]
    pub fn listening_intents(&self) -> &[ListeningIntent] {
        &self.listening_intents
    }

    #[must_use]
    pub fn stations(&self) -> &[Station] {
        &self.stations
    }

    #[must_use]
    pub fn playlists(&self) -> &[Playlist] {
        &self.playlists
    }

    #[must_use]
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub(crate) fn service(&self, id: &CatalogId) -> Option<&Service> {
        self.services.iter().find(|service| service.id() == id)
    }

    pub(crate) fn listening_intent(&self, id: &CatalogId) -> Option<&ListeningIntent> {
        self.listening_intents
            .iter()
            .find(|intent| intent.id() == id)
    }

    pub(crate) fn station(&self, id: &CatalogId) -> Option<&Station> {
        self.stations.iter().find(|station| station.id() == id)
    }

    pub(crate) fn playlist(&self, id: &CatalogId) -> Option<&Playlist> {
        self.playlists.iter().find(|playlist| playlist.id() == id)
    }

    pub(crate) fn track(&self, id: &CatalogId) -> Option<&Track> {
        self.tracks.iter().find(|track| track.id() == id)
    }

    pub(crate) fn intent_matches(&self, intent_id: &CatalogId) -> Vec<IntentMatch<'_>> {
        let Some(intent) = self.listening_intent(intent_id) else {
            return Vec::new();
        };
        let stations = intent
            .station_ids()
            .iter()
            .filter_map(|station_id| self.station(station_id).map(IntentMatch::Station));
        let playlists = intent
            .playlist_ids()
            .iter()
            .filter_map(|playlist_id| self.playlist(playlist_id).map(IntentMatch::Playlist));

        stations.chain(playlists).collect()
    }

    pub(crate) fn playlist_tracks(&self, playlist_id: &CatalogId) -> Vec<&Track> {
        self.playlist(playlist_id)
            .map_or_else(Vec::new, |playlist| {
                playlist
                    .track_ids()
                    .iter()
                    .filter_map(|track_id| self.track(track_id))
                    .collect()
            })
    }

    pub(crate) fn service_catalog(&self, service_id: &CatalogId) -> ServiceCatalogView<'_> {
        let stations = self
            .stations
            .iter()
            .filter(|station| station.source_id() == service_id)
            .map(ServiceCatalogItem::Station)
            .collect::<Vec<_>>();
        let playlists = self
            .playlists
            .iter()
            .filter(|playlist| playlist.source_id() == service_id)
            .map(ServiceCatalogItem::Playlist)
            .collect::<Vec<_>>();
        let tracks = self
            .tracks
            .iter()
            .filter(|track| track.source_id() == service_id)
            .map(ServiceCatalogItem::Track)
            .collect::<Vec<_>>();
        let counts = ServiceCatalogCounts {
            stations: stations.len(),
            playlists: playlists.len(),
            tracks: tracks.len(),
        };
        let items = stations
            .into_iter()
            .chain(playlists)
            .chain(tracks)
            .collect();

        ServiceCatalogView { items, counts }
    }

    pub(crate) fn search(&self, query: &str) -> SearchResults<'_> {
        let normalized_query = query
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if normalized_query.is_empty() {
            return SearchResults { items: Vec::new() };
        }
        let tokens = normalized_query.split_whitespace().collect::<Vec<_>>();

        let listening_intents = ranked_search_results(
            &self.listening_intents,
            |intent| {
                search_rank(
                    intent.name(),
                    &[intent.name(), intent.description()],
                    &normalized_query,
                    &tokens,
                )
            },
            SearchResult::ListeningIntent,
        );
        let stations = ranked_search_results(
            &self.stations,
            |station| {
                let source = self.service(station.source_id());
                search_rank(
                    station.name(),
                    &[
                        station.name(),
                        station.style(),
                        station.description(),
                        source.map_or("", Service::name),
                        source.map_or("", Service::badge),
                    ],
                    &normalized_query,
                    &tokens,
                )
            },
            SearchResult::Station,
        );
        let playlists = ranked_search_results(
            &self.playlists,
            |playlist| {
                let source = self.service(playlist.source_id());
                search_rank(
                    playlist.name(),
                    &[
                        playlist.name(),
                        playlist.description(),
                        source.map_or("", Service::name),
                        source.map_or("", Service::badge),
                    ],
                    &normalized_query,
                    &tokens,
                )
            },
            SearchResult::Playlist,
        );
        let tracks = ranked_search_results(
            &self.tracks,
            |track| {
                let source = self.service(track.source_id());
                search_rank(
                    track.name(),
                    &[
                        track.name(),
                        track.creator(),
                        source.map_or("", Service::name),
                        source.map_or("", Service::badge),
                    ],
                    &normalized_query,
                    &tokens,
                )
            },
            SearchResult::Track,
        );

        let items = listening_intents
            .into_iter()
            .chain(stations)
            .chain(playlists)
            .chain(tracks)
            .collect();
        SearchResults { items }
    }

    pub(crate) fn source_badge(&self, source_id: &CatalogId) -> &str {
        self.service(source_id).map_or("UNKNOWN", Service::badge)
    }

    pub(crate) fn source_name(&self, source_id: &CatalogId) -> &str {
        self.service(source_id)
            .map_or("Unknown Source", Service::name)
    }
}

fn ranked_search_results<'a, T>(
    items: &'a [T],
    rank: impl Fn(&T) -> Option<u8>,
    result: impl Fn(&'a T) -> SearchResult<'a>,
) -> Vec<SearchResult<'a>> {
    let mut ranked = items
        .iter()
        .filter_map(|item| rank(item).map(|rank| (rank, result(item))))
        .collect::<Vec<_>>();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.into_iter().map(|(_, result)| result).collect()
}

fn search_rank(
    title: &str,
    searchable_fields: &[&str],
    normalized_query: &str,
    tokens: &[&str],
) -> Option<u8> {
    let normalized_fields = searchable_fields
        .iter()
        .map(|field| field.to_lowercase())
        .collect::<Vec<_>>();
    if !tokens
        .iter()
        .all(|token| normalized_fields.iter().any(|field| field.contains(token)))
    {
        return None;
    }

    let normalized_title = title.to_lowercase();
    Some(if normalized_title == normalized_query {
        0
    } else if normalized_title.starts_with(normalized_query) {
        1
    } else {
        2
    })
}
