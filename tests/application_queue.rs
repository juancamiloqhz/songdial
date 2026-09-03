use songdial::{
    Application, DemoCatalog, Effect, Event, Key, ListeningIntent, PlaybackRequest, Playlist,
    Service, Station, Track, Viewport,
};

mod support;

use support::lines;

fn queue_catalog() -> DemoCatalog {
    DemoCatalog::new(
        vec![Service::new("test-source", "Test Source", "TEST")],
        vec![ListeningIntent::new(
            "test-intent",
            "Test Intent",
            "A deterministic Queue fixture.",
            &["test-station"],
            &["test-playlist", "empty-playlist"],
        )],
        vec![Station::available(
            "test-station",
            "Test Station",
            "Continuous test signal",
            "A Station cannot enter the Queue.",
            "test-source",
        )],
        vec![
            Playlist::new(
                "test-playlist",
                "Test Playlist",
                "Four ordered Tracks.",
                "test-source",
                &["track-one", "track-two", "track-three", "track-four"],
            ),
            Playlist::new(
                "empty-playlist",
                "Empty Playlist",
                "No playable Tracks.",
                "test-source",
                &[],
            ),
        ],
        vec![
            Track::new("track-one", "Track One", "Tester", "test-source", 2),
            Track::new("track-two", "Track Two", "Tester", "test-source", 2),
            Track::unavailable(
                "track-three",
                "Track Three",
                "Tester",
                "test-source",
                2,
                "Unavailable in this fixture.",
            ),
            Track::new("track-four", "Track Four", "Tester", "test-source", 2),
        ],
    )
}

fn expect_load(effect: Effect) -> PlaybackRequest {
    match effect {
        Effect::LoadPlayback(request) => request,
        other => panic!("expected a playback load, got {other:?}"),
    }
}

fn select_test_playlist(application: &mut Application) {
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Down));
}

fn open_test_playlist(application: &mut Application) {
    select_test_playlist(application);
    application.handle_event(Event::Key(Key::Enter));
}

fn start_test_playlist(application: &mut Application) {
    select_test_playlist(application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
}

#[test]
fn adding_a_selected_track_appends_without_interrupting_the_current_track() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('a'))),
        Effect::None
    );

    let rendered = lines(&application.render());
    assert_eq!(
        [20, 21, 22, 23].map(|row| rendered[row].trim_end().to_owned()),
        [
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
            "              Queue 1 Track",
            " QUEUE  Added Track Two to Queue. Queue 1 Track",
            "",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn successful_queue_feedback_is_brief_and_non_modal() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Char('a')));

    application.handle_event(Event::Tick);

    let rendered = lines(&application.render());
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:01/00:02",
            "              Queue 1 Track",
        )
    );
}

#[test]
fn queue_actions_are_discoverable_and_station_details_explain_the_add_limit() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    let station_guide = lines(&application.render());
    application.handle_event(Event::Key(Key::Enter));
    let station_details = lines(&application.render());
    application.handle_event(Event::Key(Key::Escape));

    application.handle_event(Event::Key(Key::Down));
    let playlist_guide = lines(&application.render());
    application.handle_event(Event::Key(Key::Enter));
    let track_guide = lines(&application.render());

    application.handle_event(Event::Key(Key::Char('?')));
    let help = lines(&application.render());

    assert!(!station_guide[23].contains("a "));
    assert_eq!(
        station_details[14].trim_end(),
        "  Queue  Stations are continuous and cannot be added."
    );
    assert!(playlist_guide[23].contains("a add"));
    assert!(track_guide[23].contains("a add"));
    assert!(track_guide[23].contains("n queue"));
    assert_eq!(
        [9, 10, 11, 12, 13, 14, 15, 16].map(|row| help[row].trim_end().to_owned()),
        [
            "  a        Add a Track or Playlist to the Queue",
            "  d        Remove the selected queued Track",
            "  /        Open Search or focus its query",
            "  n        Open Now Playing and Queue",
            "  Esc      Go back or close help",
            "  ?        Show contextual help",
            "  q / Ctrl+C  Quit",
            "  Playlist details: browse Tracks; Esc restores the prior snapshot.",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn adding_a_selected_playlist_appends_only_its_playable_tracks_in_order() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Escape));

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('a'))),
        Effect::None
    );

    let rendered = lines(&application.render());
    assert_eq!(
        [20, 21, 22].map(|row| rendered[row].trim_end().to_owned()),
        [
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
            "              Queue 3 Tracks",
            " QUEUE  Added 3 Tracks from Test Playlist. Queue 3 Tracks",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn adding_a_selected_station_is_rejected_without_interrupting_playback() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Up));

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('a'))),
        Effect::None
    );

    let rendered = lines(&application.render());
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
            "              Cannot add: Stations are continuous and cannot be queued.",
        )
    );
}

#[test]
fn adding_without_a_playback_session_explains_how_to_create_a_queue() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);

    application.handle_event(Event::Key(Key::Char('a')));

    let rendered = lines(&application.render());
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Nothing playing",
            "              Cannot add: Start a Track or Playlist before building a Queue.",
        )
    );
}

#[test]
fn adding_tracks_while_a_station_is_live_preserves_its_empty_queue() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Key(Key::Char('a')));

    let rendered = lines(&application.render());
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Test Station [TEST] • LIVE",
            "              Cannot add: Continuous Stations stay LIVE with an empty Queue.",
        )
    );
}

#[test]
fn an_unavailable_track_cannot_be_added_and_preserves_its_reason() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Key(Key::Char('a')));

    let rendered = lines(&application.render());
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
            "              Cannot add: Unavailable in this fixture.",
        )
    );
}

#[test]
fn an_empty_playlist_cannot_be_added_and_explains_why() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Key(Key::Char('a')));

    let rendered = lines(&application.render());
    assert_eq!(
        (rendered[20].trim_end(), rendered[21].trim_end()),
        (
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
            "              Cannot add: Empty Playlist has no playable Tracks.",
        )
    );
}

#[test]
fn now_playing_opens_as_a_destination_with_the_current_track_and_queue() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);

    application.handle_event(Event::Key(Key::Char('n')));

    let rendered = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 5, 6, 7, 8, 20, 21, 22, 23].map(|row| rendered[row].trim_end().to_owned()),
        [
            " SONGDIAL / NOW PLAYING",
            "  NOW PLAYING & QUEUE",
            "  CURRENT  Track One [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • 3 Tracks • Track 1/3",
            "  SELECTED > TRACK   Track Two                   Tester                   [TEST]",
            "  UNAVAIL !  TRACK   Track Three                 Tester                   [TEST]",
            "             TRACK   Track Four                  Tester                   [TEST]",
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
            "              Queue 3 Tracks",
            " ↑/k ↓/j move  Enter inspect",
            " p play  d remove  Esc back",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn queued_track_details_open_without_playback_and_back_restores_the_queue_snapshot() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Key(Key::Down));
    let queue_snapshot = application.render();

    application.handle_event(Event::Key(Key::Enter));

    let details = lines(&application.render());
    assert_eq!(
        [0, 2, 4, 5, 6, 8, 10, 11, 20].map(|row| details[row].trim_end().to_owned()),
        [
            " SONGDIAL / NOW PLAYING / TRACK THREE",
            "  TRACK",
            "  Track Three",
            "  Tester",
            "  Source  Test Source",
            "  Duration  00:02",
            "  Status  Unavailable",
            "  Unavailable in this fixture.",
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
        ]
        .map(str::to_owned)
    );

    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), queue_snapshot);
}

#[test]
fn removing_the_first_queued_track_keeps_the_current_track_playing() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));

    application.handle_event(Event::Key(Key::Char('d')));

    let rendered = lines(&application.render());
    assert_eq!(
        [3, 5, 6, 7, 20, 21, 22].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  CURRENT  Track One [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • 2 Tracks • Track 1/2",
            "  SEL+UNAV > TRACK   Track Three                 Tester                   [TEST]",
            "             TRACK   Track Four                  Tester                   [TEST]",
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
            "              Queue 2 Tracks",
            " QUEUE  Removed Track Two from Queue. Queue 2 Tracks",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn removing_a_middle_queued_track_selects_the_track_that_followed_it() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Key(Key::Char('d')));

    let rendered = lines(&application.render());
    assert_eq!(
        [5, 6, 7, 21, 22].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  QUEUE • 2 Tracks • Track 2/2",
            "             TRACK   Track Two                   Tester                   [TEST]",
            "  SELECTED > TRACK   Track Four                  Tester                   [TEST]",
            "              Queue 2 Tracks",
            " QUEUE  Removed Track Three from Queue. Queue 2 Tracks",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn removing_the_last_queued_track_moves_selection_to_the_new_last_track() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Key(Key::Char('d')));

    let rendered = lines(&application.render());
    assert_eq!(
        [5, 6, 7, 21, 22].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  QUEUE • 2 Tracks • Track 2/2",
            "             TRACK   Track Two                   Tester                   [TEST]",
            "  SEL+UNAV > TRACK   Track Three                 Tester                   [TEST]",
            "              Queue 2 Tracks",
            " QUEUE  Removed Track Four from Queue. Queue 2 Tracks",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn an_empty_track_queue_explains_how_to_add_tracks() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    open_test_playlist(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    application.handle_event(Event::Key(Key::Char('n')));

    let rendered = lines(&application.render());
    assert_eq!(
        [3, 5, 7, 8, 22, 23].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  CURRENT  Track One [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • Empty",
            "  Queue is empty.",
            "  Browse Tracks or Playlists and press a to add them.",
            " Space pause  Esc back  ? help  q quit",
            "",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn a_station_remains_live_and_explains_why_its_queue_is_empty() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Tick);

    let rendered = lines(&application.render());
    assert_eq!(
        [3, 5, 7, 8, 20, 21].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  CURRENT  Test Station [TEST] • LIVE",
            "  QUEUE • Empty",
            "  Continuous Stations remain LIVE with an empty Queue.",
            "  Play a Track or Playlist to replace this Station.",
            " NOW PLAYING  Test Station [TEST] • LIVE",
            "              Continuous Station • Queue empty",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn now_playing_without_a_session_has_actionable_empty_queue_guidance() {
    let mut application = Application::new(Viewport::new(80, 24));

    application.handle_event(Event::Key(Key::Char('n')));

    let rendered = lines(&application.render());
    assert_eq!(
        [0, 3, 5, 7, 8, 22, 23].map(|row| rendered[row].trim_end().to_owned()),
        [
            " SONGDIAL / NOW PLAYING",
            "  CURRENT  Nothing playing",
            "  QUEUE • Empty",
            "  No Playback session is active.",
            "  Start a Track or Playlist, then press a to add more Tracks.",
            " Esc back  ? help  q quit",
            "",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn playing_a_queued_track_makes_it_current_and_preserves_following_tracks() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    let while_loading = lines(&application.render());
    assert_eq!(
        (request.target(), while_loading[20].trim_end()),
        (
            &songdial::PlaybackTarget::Track(songdial::CatalogId::new("track-two")),
            " NOW PLAYING  Track One [TEST] • PLAYING • 00:00/00:02",
        )
    );

    application.handle_event(Event::PlaybackLoaded(request.id()));

    let rendered = lines(&application.render());
    assert_eq!(
        [3, 5, 6, 7, 20, 21].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  CURRENT  Track Two [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • 2 Tracks • Track 1/2",
            "  SEL+UNAV > TRACK   Track Three                 Tester                   [TEST]",
            "             TRACK   Track Four                  Tester                   [TEST]",
            " NOW PLAYING  Track Two [TEST] • PLAYING • 00:00/00:02",
            "              Queue 2 Tracks",
        ]
        .map(str::to_owned)
    );
}

fn start_deep_work_rotation(application: &mut Application) {
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
}

fn select_a_non_first_queued_track_with_a_following_track(application: &mut Application) {
    start_test_playlist(application);
    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Char('d')));
    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Char('a')));
    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Key(Key::Down));
}

#[test]
fn playing_a_non_first_queued_track_resets_focus_to_its_following_queue() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    select_a_non_first_queued_track_with_a_following_track(&mut application);

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));

    let rendered = lines(&application.render());
    assert_eq!(
        [3, 5, 6].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  CURRENT  Track Four [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • 1 Track • Track 1/1",
            "  SELECTED > TRACK   Track Two                   Tester                   [TEST]",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn queued_playback_resets_saved_queue_focus_when_details_are_open_during_load() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    select_a_non_first_queued_track_with_a_following_track(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::Key(Key::Enter));

    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Escape));

    let rendered = lines(&application.render());
    assert_eq!(
        [3, 5, 6].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  CURRENT  Track Four [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • 1 Track • Track 1/1",
            "  SELECTED > TRACK   Track Two                   Tester                   [TEST]",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn a_long_queue_scrolls_without_overwriting_the_persistent_frame_rails() {
    let mut application = Application::new(Viewport::new(80, 24));
    start_deep_work_rotation(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));
    for _ in 0..30 {
        application.handle_event(Event::Key(Key::Down));
    }

    let compact = lines(&application.render());
    assert_eq!(
        (
            compact[0].trim_end(),
            compact[5].trim_end(),
            compact[6].contains("Quiet Index"),
            compact[19].contains("SELECTED > TRACK"),
            compact[19].contains("Cinder Lines"),
            compact[20].trim_end(),
            compact[21].trim_end(),
            compact[22].trim_end(),
            compact[23].trim_end(),
        ),
        (
            " SONGDIAL / NOW PLAYING",
            "  QUEUE • 19 Tracks • Track 19/19",
            true,
            true,
            true,
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 19 Tracks",
            " ↑/k ↓/j move  Enter inspect",
            " p play  d remove  Esc back",
        )
    );

    application.handle_event(Event::Resize(Viewport::new(120, 40)));
    let wide = lines(&application.render());
    assert_eq!(
        (
            wide[0].trim_end(),
            wide[5].trim_end(),
            wide[36].trim_end(),
            wide[37].trim_end(),
            wide[38].trim_end(),
            wide[39].trim_end(),
        ),
        (
            " SONGDIAL / NOW PLAYING",
            "  QUEUE • 19 Tracks • Track 19/19",
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 19 Tracks",
            " ↑/k ↓/j move  Enter inspect",
            " p play  d remove  Esc back",
        )
    );
}

#[test]
fn a_wide_queue_uses_the_available_browser_height_without_changing_keys() {
    let mut application = Application::new(Viewport::new(120, 40));
    start_deep_work_rotation(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));
    for _ in 0..30 {
        application.handle_event(Event::Key(Key::Down));
    }

    let rendered = lines(&application.render());
    assert_eq!(
        (
            rendered[0].trim_end(),
            rendered[5].trim_end(),
            rendered[6].contains("Night Geometry"),
            rendered[6].contains("[HARBOR]"),
            rendered[24].contains("SELECTED > TRACK"),
            rendered[24].contains("Cinder Lines"),
            rendered[36].trim_end(),
            rendered[38].trim_end(),
        ),
        (
            " SONGDIAL / NOW PLAYING",
            "  QUEUE • 19 Tracks • Track 19/19",
            true,
            true,
            true,
            true,
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            " ↑/k ↓/j move  Enter inspect",
        )
    );
}

#[test]
fn back_from_now_playing_restores_the_exact_prior_browse_snapshot() {
    let mut application = Application::new(Viewport::new(80, 24));
    start_deep_work_rotation(&mut application);
    let browse_rows = lines(&application.render())[..20].to_vec();

    application.handle_event(Event::Key(Key::Char('n')));
    for _ in 0..10 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Char('d')));
    application.handle_event(Event::Key(Key::Escape));

    assert_eq!(lines(&application.render())[..20], browse_rows);
}

#[test]
fn automatic_advancement_preserves_queue_focus_and_retains_the_final_track_stopped() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Tick);
    application.handle_event(Event::Tick);

    let first_advance = lines(&application.render());
    assert_eq!(
        [3, 5, 6, 7].map(|row| first_advance[row].trim_end().to_owned()),
        [
            "  CURRENT  Track Two [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • 2 Tracks • Track 2/2",
            "  UNAVAIL !  TRACK   Track Three                 Tester                   [TEST]",
            "  SELECTED > TRACK   Track Four                  Tester                   [TEST]",
        ]
        .map(str::to_owned)
    );

    application.handle_event(Event::Tick);
    application.handle_event(Event::Tick);
    let final_playing = lines(&application.render());
    assert_eq!(
        (final_playing[3].trim_end(), final_playing[5].trim_end()),
        (
            "  CURRENT  Track Four [TEST] • PLAYING • 00:00/00:02",
            "  QUEUE • Empty",
        )
    );

    application.handle_event(Event::Tick);
    application.handle_event(Event::Tick);
    assert_eq!(
        lines(&application.render())[3].trim_end(),
        "  CURRENT  Track Four [TEST] • STOPPED • 00:02/00:02"
    );
}

#[test]
fn automatic_advancement_keeps_the_saved_queue_focus_valid_while_details_are_open() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), queue_catalog());
    start_test_playlist(&mut application);
    application.handle_event(Event::Key(Key::Char('n')));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));

    application.handle_event(Event::Tick);
    application.handle_event(Event::Tick);
    application.handle_event(Event::Key(Key::Escape));

    let rendered = lines(&application.render());
    assert_eq!(
        [5, 6, 7].map(|row| rendered[row].trim_end().to_owned()),
        [
            "  QUEUE • 2 Tracks • Track 2/2",
            "  UNAVAIL !  TRACK   Track Three                 Tester                   [TEST]",
            "  SELECTED > TRACK   Track Four                  Tester                   [TEST]",
        ]
        .map(str::to_owned)
    );
}
