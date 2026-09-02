use songdial::{
    Application, CatalogId, DemoCatalog, Effect, Event, Key, ListeningIntent, PlaybackRequest,
    PlaybackTarget, Playlist, Service, Track, Viewport,
};

mod support;

use support::lines;

fn open_deep_work(application: &mut Application) {
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
}

fn expect_load(effect: Effect) -> PlaybackRequest {
    match effect {
        Effect::LoadPlayback(request) => request,
        other => panic!("expected a playback load, got {other:?}"),
    }
}

fn short_playlist_catalog() -> DemoCatalog {
    DemoCatalog::new(
        vec![Service::new("test-source", "Test Source", "TEST")],
        vec![ListeningIntent::new(
            "test-intent",
            "Test Intent",
            "A deterministic test fixture.",
            &[],
            &["short-playlist"],
        )],
        vec![],
        vec![Playlist::new(
            "short-playlist",
            "Short Playlist",
            "Two short Tracks.",
            "test-source",
            &["track-one", "track-two"],
        )],
        vec![
            Track::new("track-one", "Track One", "Tester", "test-source", 2),
            Track::new("track-two", "Track Two", "Tester", "test-source", 1),
        ],
    )
}

fn playlist_with_unavailable_first_track() -> DemoCatalog {
    DemoCatalog::new(
        vec![Service::new("test-source", "Test Source", "TEST")],
        vec![ListeningIntent::new(
            "test-intent",
            "Test Intent",
            "A deterministic test fixture.",
            &[],
            &["available-playlist"],
        )],
        vec![],
        vec![Playlist::new(
            "available-playlist",
            "Available Playlist",
            "Starts at the first playable Track.",
            "test-source",
            &["unavailable-track", "available-track", "last-track"],
        )],
        vec![
            Track::unavailable(
                "unavailable-track",
                "Unavailable Track",
                "Tester",
                "test-source",
                1,
                "Not available.",
            ),
            Track::new(
                "available-track",
                "Available Track",
                "Tester",
                "test-source",
                1,
            ),
            Track::new("last-track", "Last Track", "Tester", "test-source", 1),
        ],
    )
}

#[test]
fn station_loads_as_live_and_persists_while_browsing() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Station(CatalogId::new("morrow-night-ledger"))
    );
    let loading = lines(&application.render());
    assert_eq!(
        (
            loading[20].trim_end(),
            loading[21].trim_end(),
            loading[22].trim_end(),
        ),
        (
            " NOW PLAYING  Loading Night Ledger [MORROW]",
            "              Waiting for simulated playback.",
            " ↑/k up  ↓/j down  Enter inspect  p play",
        )
    );

    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Escape));

    let playing = lines(&application.render());
    assert_eq!(
        (
            playing[0].trim_end(),
            playing[20].trim_end(),
            playing[21].trim_end()
        ),
        (
            " SONGDIAL / MOOD & ACTIVITY",
            " NOW PLAYING  Night Ledger [MORROW] • LIVE",
            "              Continuous Station • Queue empty",
        )
    );
}

#[test]
fn playlist_starts_its_first_track_and_queues_the_remainder() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Track(CatalogId::new("morrow-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(request.id()));

    let playing = lines(&application.render());
    assert_eq!(
        (playing[20].trim_end(), playing[21].trim_end()),
        (
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 19 Tracks",
        )
    );
}

#[test]
fn track_starts_with_an_empty_queue_and_keeps_navigation_independent() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Track(CatalogId::new("morrow-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Escape));

    let rendered = lines(&application.render());
    assert!(rendered[18].contains("SELECTED > PLAYLIST"));
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 0 Tracks",
        )
    );
}

#[test]
fn track_ticks_are_deterministic_and_pause_resume_is_immediate() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    application.handle_event(Event::Tick);
    application.handle_event(Event::Tick);
    assert!(lines(&application.render())[20].contains("PLAYING • 00:02/05:28"));

    application.handle_event(Event::Key(Key::Char(' ')));
    application.handle_event(Event::Tick);
    assert!(lines(&application.render())[20].contains("PAUSED • 00:02/05:28"));

    application.handle_event(Event::Key(Key::Char(' ')));
    assert!(lines(&application.render())[20].contains("PLAYING • 00:02/05:28"));
}

#[test]
fn playlist_advances_then_retains_and_restarts_its_final_stopped_track() {
    let mut application =
        Application::with_catalog(Viewport::new(80, 24), short_playlist_catalog());
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    application.handle_event(Event::Tick);
    application.handle_event(Event::Tick);
    let advanced = lines(&application.render());
    assert_eq!(
        (advanced[20].trim_end(), advanced[21].trim_end()),
        (
            " NOW PLAYING  Track Two [TEST] • PLAYING • 00:00/00:01",
            "              Queue 0 Tracks",
        )
    );

    application.handle_event(Event::Tick);
    assert!(lines(&application.render())[20].contains("STOPPED • 00:01/00:01"));

    application.handle_event(Event::Key(Key::Char(' ')));
    assert!(lines(&application.render())[20].contains("PLAYING • 00:00/00:01"));
}

#[test]
fn station_can_pause_and_resume_without_ever_finishing() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    application.handle_event(Event::Key(Key::Char(' ')));
    application.handle_event(Event::Tick);
    assert!(lines(&application.render())[20].contains("LIVE • PAUSED"));

    application.handle_event(Event::Key(Key::Char(' ')));
    application.handle_event(Event::Tick);
    assert!(
        lines(&application.render())[20]
            .trim_end()
            .ends_with("• LIVE")
    );
}

#[test]
fn failed_replacement_keeps_the_existing_session_and_explains_the_failure() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    let first = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(first.id()));

    application.handle_event(Event::Key(Key::Down));
    let replacement = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    let loading = lines(&application.render());
    assert_eq!(
        (loading[20].trim_end(), loading[21].trim_end()),
        (
            " NOW PLAYING  Night Ledger [MORROW] • LIVE",
            "              Loading Daylight Circuit [HARBOR]",
        )
    );

    application.handle_event(Event::PlaybackFailed {
        request_id: replacement.id(),
        reason: "The simulated signal could not load.".to_owned(),
    });
    let failed = lines(&application.render());
    assert_eq!(
        (failed[20].trim_end(), failed[21].trim_end()),
        (
            " NOW PLAYING  Night Ledger [MORROW] • LIVE",
            "              Playback failed: The simulated signal could not load.",
        )
    );
}

#[test]
fn only_the_latest_overlapping_playback_request_can_commit() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    let original = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(original.id()));

    application.handle_event(Event::Key(Key::Down));
    let stale = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::Key(Key::Down));
    let latest = expect_load(application.handle_event(Event::Key(Key::Char('p'))));

    application.handle_event(Event::PlaybackLoaded(stale.id()));
    application.handle_event(Event::PlaybackFailed {
        request_id: stale.id(),
        reason: "A stale failure.".to_owned(),
    });
    let still_loading = lines(&application.render());
    assert_eq!(
        (still_loading[20].trim_end(), still_loading[21].trim_end()),
        (
            " NOW PLAYING  Night Ledger [MORROW] • LIVE",
            "              Loading Stillwater FM [MORROW]",
        )
    );

    application.handle_event(Event::PlaybackLoaded(latest.id()));
    let replaced = lines(&application.render());
    assert_eq!(
        (replaced[20].trim_end(), replaced[21].trim_end()),
        (
            " NOW PLAYING  Stillwater FM [MORROW] • LIVE",
            "              Continuous Station • Queue empty",
        )
    );
}

#[test]
fn empty_playlist_cannot_start_and_explains_why() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..12 {
        application.handle_event(Event::Key(Key::Down));
    }

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    let rendered = lines(&application.render());
    assert_eq!(
        rendered[21].trim_end(),
        "              Cannot play: Empty Room has no Tracks."
    );
}

#[test]
fn unavailable_station_cannot_start_and_preserves_its_reason() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..6 {
        application.handle_event(Event::Key(Key::Down));
    }

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    application.handle_event(Event::Key(Key::Escape));
    let rendered = lines(&application.render());
    assert_eq!(
        rendered[21].trim_end(),
        "              Cannot play: Signal maintenance is in progress."
    );
}

#[test]
fn unavailable_track_cannot_start_and_preserves_its_reason() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..15 {
        application.handle_event(Event::Key(Key::Down));
    }

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    application.handle_event(Event::Key(Key::Escape));
    let rendered = lines(&application.render());
    assert_eq!(
        rendered[21].trim_end(),
        "              Cannot play: This recording is unavailable in the Demo catalog."
    );
}

#[test]
fn playlist_starts_at_its_first_available_track() {
    let mut application = Application::with_catalog(
        Viewport::new(80, 24),
        playlist_with_unavailable_first_track(),
    );
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Track(CatalogId::new("available-track"))
    );
    application.handle_event(Event::PlaybackLoaded(request.id()));
    let rendered = lines(&application.render());
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Available Track [TEST] • PLAYING • 00:00/00:01",
            "              Queue 1 Track",
        )
    );
}

#[test]
fn browser_text_keeps_playing_and_selection_states_independent() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    let combined = lines(&application.render());
    assert!(combined[6].contains("SEL+PLAY > STATION   Night Ledger"));

    application.handle_event(Event::Key(Key::Down));
    let independent = lines(&application.render());
    assert!(independent[6].contains("PLAYING *  STATION   Night Ledger"));
    assert!(independent[8].contains("SELECTED > STATION   Daylight Circuit"));
    assert!(independent[20].contains("Night Ledger [MORROW] • LIVE"));
}

#[test]
fn track_rows_show_combined_and_independent_playback_state_in_text() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    let combined = lines(&application.render());
    assert!(combined[7].contains("SEL+PLAY > TRACK   Night Geometry"));

    application.handle_event(Event::Key(Key::Down));
    let independent = lines(&application.render());
    assert!(independent[7].contains("PLAYING *  TRACK   Night Geometry"));
    assert!(independent[8].contains("SELECTED > TRACK   Night Geometry"));
}

#[test]
fn playback_keys_are_discoverable_in_context_and_complete_help() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    application.handle_event(Event::Key(Key::Enter));
    let details = lines(&application.render());
    assert!(details[22].contains("p play"));
    assert!(details[22].contains("Space pause"));

    application.handle_event(Event::Key(Key::Char('?')));
    let help = lines(&application.render());
    assert_eq!(
        (help[7].trim_end(), help[8].trim_end()),
        (
            "  p        Start a new Playback session",
            "  Space    Pause, resume, or restart",
        )
    );
}

#[test]
fn playlist_rows_retain_the_playback_origin_state_in_text() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_deep_work(&mut application);
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    let combined = lines(&application.render());
    assert!(combined[18].contains("SEL+PLAY > PLAYLIST  Deep Work Rotation"));

    application.handle_event(Event::Key(Key::Down));
    let independent = lines(&application.render());
    assert!(independent[16].contains("PLAYING *  PLAYLIST  Deep Work Rotation"));
    assert!(independent[18].contains("SELECTED > PLAYLIST  Focus Lines"));
}
