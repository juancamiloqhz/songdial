#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CatalogId(&'static str);

impl CatalogId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Availability {
    Available,
    Loading,
    Unavailable(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CatalogIdentity {
    id: CatalogId,
    name: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Service {
    identity: CatalogIdentity,
    badge: &'static str,
}

impl Service {
    const fn new(id: &'static str, name: &'static str, badge: &'static str) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId(id),
                name,
            },
            badge,
        }
    }

    #[must_use]
    pub const fn id(&self) -> CatalogId {
        self.identity.id
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.identity.name
    }

    #[must_use]
    pub const fn badge(&self) -> &'static str {
        self.badge
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListeningIntent {
    identity: CatalogIdentity,
    description: &'static str,
    station_ids: Vec<CatalogId>,
    playlist_ids: Vec<CatalogId>,
}

impl ListeningIntent {
    fn new(
        id: &'static str,
        name: &'static str,
        description: &'static str,
        station_ids: &[&'static str],
        playlist_ids: &[&'static str],
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId(id),
                name,
            },
            description,
            station_ids: station_ids.iter().copied().map(CatalogId).collect(),
            playlist_ids: playlist_ids.iter().copied().map(CatalogId).collect(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> CatalogId {
        self.identity.id
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.identity.name
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        self.description
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
    style: &'static str,
    description: &'static str,
    source_id: CatalogId,
    availability: Availability,
}

impl Station {
    const fn new(
        id: &'static str,
        name: &'static str,
        style: &'static str,
        description: &'static str,
        source_id: &'static str,
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

    const fn loading(
        id: &'static str,
        name: &'static str,
        style: &'static str,
        description: &'static str,
        source_id: &'static str,
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

    const fn unavailable(
        id: &'static str,
        name: &'static str,
        style: &'static str,
        description: &'static str,
        source_id: &'static str,
        reason: &'static str,
    ) -> Self {
        Self::with_availability(
            id,
            name,
            style,
            description,
            source_id,
            Availability::Unavailable(reason),
        )
    }

    const fn with_availability(
        id: &'static str,
        name: &'static str,
        style: &'static str,
        description: &'static str,
        source_id: &'static str,
        availability: Availability,
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId(id),
                name,
            },
            style,
            description,
            source_id: CatalogId(source_id),
            availability,
        }
    }

    #[must_use]
    pub const fn id(&self) -> CatalogId {
        self.identity.id
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.identity.name
    }

    #[must_use]
    pub const fn style(&self) -> &'static str {
        self.style
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        self.description
    }

    #[must_use]
    pub const fn source_id(&self) -> CatalogId {
        self.source_id
    }

    #[must_use]
    pub const fn availability(&self) -> Availability {
        self.availability
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Playlist {
    identity: CatalogIdentity,
    description: &'static str,
    source_id: CatalogId,
    track_ids: Vec<CatalogId>,
}

impl Playlist {
    fn new(
        id: &'static str,
        name: &'static str,
        description: &'static str,
        source_id: &'static str,
        track_ids: &[&'static str],
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId(id),
                name,
            },
            description,
            source_id: CatalogId(source_id),
            track_ids: track_ids.iter().copied().map(CatalogId).collect(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> CatalogId {
        self.identity.id
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.identity.name
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        self.description
    }

    #[must_use]
    pub const fn source_id(&self) -> CatalogId {
        self.source_id
    }

    #[must_use]
    pub const fn availability(&self) -> Availability {
        Availability::Available
    }

    #[must_use]
    pub fn track_ids(&self) -> &[CatalogId] {
        &self.track_ids
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Track {
    identity: CatalogIdentity,
    creator: &'static str,
    source_id: CatalogId,
    duration_seconds: u16,
    availability: Availability,
}

impl Track {
    const fn new(
        id: &'static str,
        name: &'static str,
        creator: &'static str,
        source_id: &'static str,
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

    const fn unavailable(
        id: &'static str,
        name: &'static str,
        creator: &'static str,
        source_id: &'static str,
        duration_seconds: u16,
        reason: &'static str,
    ) -> Self {
        Self::with_availability(
            id,
            name,
            creator,
            source_id,
            duration_seconds,
            Availability::Unavailable(reason),
        )
    }

    const fn with_availability(
        id: &'static str,
        name: &'static str,
        creator: &'static str,
        source_id: &'static str,
        duration_seconds: u16,
        availability: Availability,
    ) -> Self {
        Self {
            identity: CatalogIdentity {
                id: CatalogId(id),
                name,
            },
            creator,
            source_id: CatalogId(source_id),
            duration_seconds,
            availability,
        }
    }

    #[must_use]
    pub const fn id(&self) -> CatalogId {
        self.identity.id
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.identity.name
    }

    #[must_use]
    pub const fn creator(&self) -> &'static str {
        self.creator
    }

    #[must_use]
    pub const fn source_id(&self) -> CatalogId {
        self.source_id
    }

    #[must_use]
    pub const fn duration_seconds(&self) -> u16 {
        self.duration_seconds
    }

    #[must_use]
    pub const fn availability(&self) -> Availability {
        self.availability
    }
}

/// The fixed fictional catalog used by the first Songdial milestone.
///
/// This data is deliberately local and deterministic. It is not a schema for
/// future Service integrations.
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
                Station::new(
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
                Station::new(
                    "morrow-stillwater-fm",
                    "Stillwater FM",
                    "Ambient piano",
                    "Sparse piano and room tone with long breathing space.",
                    "morrow-audio",
                ),
                Station::new(
                    "harbor-kinetic-line",
                    "Kinetic Line",
                    "Instrumental pulse",
                    "Measured rhythmic instrumentals with no vocals.",
                    "harbor-sound",
                ),
                Station::new(
                    "morrow-low-tide-radio",
                    "Low Tide Radio",
                    "Downtempo",
                    "Slow electronic currents for a settled pace.",
                    "morrow-audio",
                ),
                Station::new(
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
                Station::new(
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
}
