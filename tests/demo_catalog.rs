use songdial::{Availability, CatalogId, DemoCatalog};

fn ids(identifiers: impl IntoIterator<Item = CatalogId>) -> Vec<&'static str> {
    identifiers.into_iter().map(CatalogId::as_str).collect()
}

#[test]
fn fixed_demo_catalog_has_required_entity_counts_and_stable_order() {
    let catalog = DemoCatalog::fixed();

    assert_eq!(
        (
            ids(catalog.services().iter().map(|service| service.id())),
            ids(catalog.listening_intents().iter().map(|intent| intent.id())),
            ids(catalog.stations().iter().map(|station| station.id())),
            ids(catalog.playlists().iter().map(|playlist| playlist.id())),
            ids(catalog.tracks().iter().map(|track| track.id())),
        ),
        (
            vec!["morrow-audio", "harbor-sound"],
            vec!["deep-work", "focus", "flow", "calm", "energy", "reset"],
            vec![
                "morrow-night-ledger",
                "harbor-daylight-circuit",
                "morrow-stillwater-fm",
                "harbor-kinetic-line",
                "morrow-low-tide-radio",
                "harbor-afterglow-signal",
                "morrow-northbound-static",
                "harbor-open-frequency",
            ],
            vec![
                "morrow-deep-work-rotation",
                "harbor-focus-lines",
                "morrow-calm-current",
                "harbor-energy-shift",
                "harbor-empty-room",
            ],
            vec![
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
                "morrow-archive-of-air",
                "harbor-bright-current",
                "morrow-long-form",
                "harbor-return-path",
            ],
        )
    );
}

#[test]
fn fixed_demo_catalog_has_empty_and_long_playlist_scenarios() {
    let catalog = DemoCatalog::fixed();
    let playlist_lengths = catalog
        .playlists()
        .iter()
        .map(|playlist| (playlist.id().as_str(), playlist.track_ids().len()))
        .collect::<Vec<_>>();

    assert_eq!(
        playlist_lengths,
        [
            ("morrow-deep-work-rotation", 20),
            ("harbor-focus-lines", 4),
            ("morrow-calm-current", 6),
            ("harbor-energy-shift", 5),
            ("harbor-empty-room", 0),
        ]
    );
}

#[test]
fn duplicate_recordings_remain_source_scoped_to_fictional_services() {
    let catalog = DemoCatalog::fixed();
    let services = catalog
        .services()
        .iter()
        .map(|service| (service.id().as_str(), service.name()))
        .collect::<Vec<_>>();
    let mut duplicates = Vec::new();

    for (index, track) in catalog.tracks().iter().enumerate() {
        for duplicate in &catalog.tracks()[index + 1..] {
            if track.name() == duplicate.name() && track.creator() == duplicate.creator() {
                duplicates.push((
                    track.name(),
                    track.creator(),
                    track.source_id().as_str(),
                    duplicate.source_id().as_str(),
                ));
            }
        }
    }

    assert_eq!(
        (services, duplicates),
        (
            vec![
                ("morrow-audio", "Morrow Audio"),
                ("harbor-sound", "Harbor Sound"),
            ],
            vec![
                (
                    "Night Geometry",
                    "Sable Circuit",
                    "morrow-audio",
                    "harbor-sound",
                ),
                (
                    "Stillwater Signal",
                    "Vale Static",
                    "morrow-audio",
                    "harbor-sound",
                ),
            ],
        )
    );
}

#[test]
fn fixed_demo_catalog_keeps_loading_and_unavailable_scenarios_deterministic() {
    let catalog = DemoCatalog::fixed();
    let station_scenarios = catalog
        .stations()
        .iter()
        .filter(|station| station.availability() != Availability::Available)
        .map(|station| (station.id().as_str(), station.availability()))
        .collect::<Vec<_>>();
    let track_scenarios = catalog
        .tracks()
        .iter()
        .filter(|track| track.availability() != Availability::Available)
        .map(|track| (track.id().as_str(), track.availability()))
        .collect::<Vec<_>>();

    assert_eq!(
        (station_scenarios, track_scenarios),
        (
            vec![
                ("harbor-daylight-circuit", Availability::Loading),
                (
                    "morrow-northbound-static",
                    Availability::Unavailable("Signal maintenance is in progress."),
                ),
            ],
            vec![(
                "harbor-blueprint-sky",
                Availability::Unavailable("This recording is unavailable in the Demo catalog.",),
            )],
        )
    );
}

#[test]
fn listening_intent_matches_are_ordered_and_include_an_empty_collection() {
    let catalog = DemoCatalog::fixed();
    let matches = catalog
        .listening_intents()
        .iter()
        .map(|intent| {
            (
                intent.id().as_str(),
                ids(intent.station_ids().iter().copied()),
                ids(intent.playlist_ids().iter().copied()),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        matches,
        [
            (
                "deep-work",
                vec![
                    "morrow-night-ledger",
                    "harbor-daylight-circuit",
                    "morrow-stillwater-fm",
                    "harbor-kinetic-line",
                    "morrow-low-tide-radio",
                    "harbor-afterglow-signal",
                    "morrow-northbound-static",
                    "harbor-open-frequency",
                ],
                vec![
                    "morrow-deep-work-rotation",
                    "harbor-focus-lines",
                    "morrow-calm-current",
                    "harbor-energy-shift",
                    "harbor-empty-room",
                ],
            ),
            (
                "focus",
                vec![
                    "morrow-night-ledger",
                    "harbor-daylight-circuit",
                    "morrow-stillwater-fm",
                ],
                vec!["morrow-deep-work-rotation", "harbor-focus-lines"],
            ),
            (
                "flow",
                vec!["morrow-night-ledger", "harbor-kinetic-line"],
                vec!["morrow-deep-work-rotation", "harbor-energy-shift"],
            ),
            (
                "calm",
                vec![
                    "morrow-stillwater-fm",
                    "morrow-low-tide-radio",
                    "harbor-afterglow-signal",
                ],
                vec!["morrow-calm-current", "harbor-empty-room"],
            ),
            (
                "energy",
                vec![
                    "harbor-daylight-circuit",
                    "harbor-kinetic-line",
                    "harbor-open-frequency",
                ],
                vec!["harbor-energy-shift"],
            ),
            ("reset", vec![], vec![]),
        ]
    );
}

#[test]
fn track_durations_are_literal_and_stable_in_catalog_order() {
    let catalog = DemoCatalog::fixed();

    assert_eq!(
        catalog
            .tracks()
            .iter()
            .map(|track| track.duration_seconds())
            .collect::<Vec<_>>(),
        [
            328, 331, 284, 286, 302, 276, 315, 247, 293, 261, 308, 255, 322, 269, 297, 244, 311,
            278, 305, 252, 319, 263, 300, 271,
        ]
    );
}
