use songdial::{
    Application, CatalogId, DemoCatalog, Effect, Event, Key, ListeningIntent, PlaybackRequest,
    PlaybackTarget, Playlist, Service, Station, Track, Viewport,
};

mod support;

use support::lines;

fn open_search_from_home(application: &mut Application) {
    for _ in 0..4 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
}

fn type_query(application: &mut Application, query: &str) {
    for character in query.chars() {
        application.handle_event(Event::Key(Key::Char(character)));
    }
}

fn ranking_catalog() -> DemoCatalog {
    DemoCatalog::new(
        vec![Service::new("local", "Local Source", "LOCAL")],
        vec![],
        vec![],
        vec![],
        vec![
            Track::new("middle", "Midnight Signal", "Aria", "local", 60),
            Track::new("prefix", "Signal Garden", "Bela", "local", 60),
            Track::new("exact", "Signal", "Cato", "local", 60),
            Track::new("creator", "Beacon", "Signal Maker", "local", 60),
        ],
    )
}

fn grouped_catalog() -> DemoCatalog {
    DemoCatalog::new(
        vec![Service::new("local", "Local Source", "LOCAL")],
        vec![ListeningIntent::new(
            "signal-focus",
            "Signal Focus",
            "A steady choice.",
            &[],
            &[],
        )],
        vec![Station::available(
            "signal-radio",
            "Signal Radio",
            "Ambient",
            "A continuous stream.",
            "local",
        )],
        vec![Playlist::new(
            "signal-set",
            "Signal Set",
            "An ordered sequence.",
            "local",
            &["signal-tune", "quiet-echo"],
        )],
        vec![
            Track::new("signal-tune", "Signal Tune", "Aria", "local", 60),
            Track::new("quiet-echo", "Quiet Echo", "Bela", "local", 70),
        ],
    )
}

fn expect_load(effect: Effect) -> PlaybackRequest {
    match effect {
        Effect::LoadPlayback(request) => request,
        other => panic!("expected a playback load, got {other:?}"),
    }
}

#[test]
fn search_opens_with_focused_empty_query_and_escape_leaves_editing_before_going_back() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_search_from_home(&mut application);

    let initial = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 20, 22, 23].map(|row| initial[row].trim_end().to_owned()),
        [
            " SONGDIAL / SEARCH EVERYTHING",
            "  SEARCH EVERYTHING",
            "  Query >",
            "  Start typing to search the Demo catalog.",
            " NOW PLAYING  Nothing playing",
            " Type to search  Backspace erase",
            " ↑/↓ move  Enter inspect  Esc done",
        ]
        .map(str::to_owned)
    );

    type_query(&mut application, "Night");
    assert_eq!(
        lines(&application.render())[3].trim_end(),
        "  Query > Night"
    );

    application.handle_event(Event::Key(Key::Escape));
    let unfocused = lines(&application.render());
    assert_eq!(unfocused[0].trim_end(), " SONGDIAL / SEARCH EVERYTHING");
    assert_eq!(unfocused[3].trim_end(), "  Query   Night");

    application.handle_event(Event::Key(Key::Char('/')));
    assert_eq!(
        lines(&application.render())[3].trim_end(),
        "  Query > Night"
    );
    assert_eq!(
        application.handle_event(Event::Key(Key::Char('q'))),
        Effect::None
    );
    assert_eq!(
        lines(&application.render())[3].trim_end(),
        "  Query > Nightq"
    );

    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Escape));
    let home = lines(&application.render());
    assert_eq!(home[0].trim_end(), " SONGDIAL / HOME");
    assert_eq!(
        home[8].trim_end(),
        "  > Search everything                                         SELECTED"
    );
}

#[test]
fn search_case_folds_and_ranks_exact_then_prefix_then_other_matches_stably() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), ranking_catalog());
    application.handle_event(Event::Key(Key::Char('/')));
    type_query(&mut application, "SIGNAL");

    let rendered = lines(&application.render());
    let result_rows = rendered[6..12]
        .iter()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>();

    assert_eq!(
        result_rows,
        [
            "  TRACKS • 4",
            "  SELECTED > TRACK     Signal                                          [LOCAL]",
            "             TRACK     Signal Garden                                   [LOCAL]",
            "             TRACK     Midnight Signal                                 [LOCAL]",
            "             TRACK     Beacon                                          [LOCAL]",
            "",
        ]
    );
}

#[test]
fn search_groups_every_result_kind_in_order_and_omits_empty_groups() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), grouped_catalog());
    application.handle_event(Event::Key(Key::Char('/')));
    type_query(&mut application, "signal");

    let grouped = lines(&application.render());
    assert_eq!(
        [6, 7, 8, 9, 10, 11, 12, 13].map(|row| grouped[row].trim_end().to_owned()),
        [
            "  LISTENING INTENTS • 1",
            "  SELECTED > INTENT    Signal Focus",
            "  STATIONS • 1",
            "             STATION   Signal Radio                                    [LOCAL]",
            "  PLAYLISTS • 1",
            "             PLAYLIST  Signal Set                                      [LOCAL]",
            "  TRACKS • 1",
            "             TRACK     Signal Tune                                     [LOCAL]",
        ]
        .map(str::to_owned)
    );

    for _ in 0..6 {
        application.handle_event(Event::Key(Key::Backspace));
    }
    type_query(&mut application, "tune");
    let track_only = lines(&application.render());
    assert_eq!(track_only[6].trim_end(), "  TRACKS • 1");
    assert!(
        !track_only[..20]
            .iter()
            .any(|line| line.contains("LISTENING INTENTS") || line.contains("STATIONS •"))
    );

    for _ in 0..4 {
        application.handle_event(Event::Key(Key::Backspace));
    }
    type_query(&mut application, "missing");
    let missing = lines(&application.render());
    assert_eq!(missing[4].trim_end(), "  No results for “missing”.");
    assert!(missing[6..20].iter().all(|line| line.trim().is_empty()));
}

#[test]
fn duplicate_tracks_keep_their_sources_and_open_without_playing_before_deliberate_playback() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Char('/')));
    type_query(&mut application, "Night Geometry");

    let first_result_snapshot = application.render();
    let duplicate_results = lines(&first_result_snapshot);
    assert_eq!(
        [6, 7, 8].map(|row| duplicate_results[row].trim_end().to_owned()),
        [
            "  TRACKS • 2",
            "  SELECTED > TRACK     Night Geometry                                 [MORROW]",
            "             TRACK     Night Geometry                                 [HARBOR]",
        ]
        .map(str::to_owned)
    );

    assert_eq!(
        application.handle_event(Event::Key(Key::Enter)),
        Effect::None
    );
    let details = lines(&application.render());
    assert_eq!(
        [0, 2, 4, 5, 6, 12, 20].map(|row| details[row].trim_end().to_owned()),
        [
            " SONGDIAL / SEARCH EVERYTHING / NIGHT GEOMETRY",
            "  TRACK",
            "  Night Geometry",
            "  Sable Circuit",
            "  Source  Morrow Audio",
            "  Enter opened details only. Nothing started playing.",
            " NOW PLAYING  Nothing playing",
        ]
        .map(str::to_owned)
    );

    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), first_result_snapshot);

    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Down));
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Track(CatalogId::new("harbor-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(request.id()));
    assert_eq!(
        lines(&application.render())[20].trim_end(),
        " NOW PLAYING  Night Geometry [HARBOR] • PLAYING • 00:00/05:31"
    );
}

#[test]
fn every_search_result_opens_and_playable_results_use_shared_session_rules() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), grouped_catalog());
    application.handle_event(Event::Key(Key::Char('/')));
    type_query(&mut application, "signal");

    let focused_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    assert_eq!(
        lines(&application.render())[0].trim_end(),
        " SONGDIAL / MOOD & ACTIVITY / SIGNAL FOCUS"
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), focused_snapshot);

    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Down));
    let station_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    assert_eq!(
        lines(&application.render())[0].trim_end(),
        " SONGDIAL / SEARCH EVERYTHING / SIGNAL RADIO"
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), station_snapshot);
    let station_request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        station_request.target(),
        &PlaybackTarget::Station(CatalogId::new("signal-radio"))
    );
    application.handle_event(Event::PlaybackLoaded(station_request.id()));

    application.handle_event(Event::Key(Key::Down));
    let playlist_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    assert_eq!(
        lines(&application.render())[0].trim_end(),
        " SONGDIAL / SEARCH EVERYTHING / SIGNAL SET"
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), playlist_snapshot);
    let playlist_request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        playlist_request.target(),
        &PlaybackTarget::Track(CatalogId::new("signal-tune"))
    );
    application.handle_event(Event::PlaybackLoaded(playlist_request.id()));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Queue 1 Track"
    );

    application.handle_event(Event::Key(Key::Down));
    let track_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    assert_eq!(
        lines(&application.render())[0].trim_end(),
        " SONGDIAL / SEARCH EVERYTHING / SIGNAL TUNE"
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), track_snapshot);
    let track_request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        track_request.target(),
        &PlaybackTarget::Track(CatalogId::new("signal-tune"))
    );
    application.handle_event(Event::PlaybackLoaded(track_request.id()));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Queue 0 Tracks"
    );
}

#[test]
fn global_search_and_back_restore_the_exact_scrolled_query_and_parent_snapshots() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..7 {
        application.handle_event(Event::Key(Key::Down));
    }
    let stations_snapshot = application.render();

    application.handle_event(Event::Key(Key::Char('/')));
    type_query(&mut application, "MORROW");
    assert_eq!(
        lines(&application.render())[4].trim_end(),
        "  18 results • Result 1/18"
    );
    for _ in 0..12 {
        application.handle_event(Event::Key(Key::Down));
    }
    let scrolled_search_snapshot = application.render();
    let scrolled = lines(&scrolled_search_snapshot);
    assert_eq!(scrolled[4].trim_end(), "  18 results • Result 13/18");
    assert!(scrolled[16].contains("SELECTED > TRACK"));

    application.handle_event(Event::Key(Key::Enter));
    assert!(
        lines(&application.render())[0]
            .trim_end()
            .starts_with(" SONGDIAL / SEARCH EVERYTHING / ")
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), scrolled_search_snapshot);

    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), stations_snapshot);
}

#[test]
fn focused_query_updates_incrementally_while_search_and_help_keys_keep_their_meaning() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), ranking_catalog());
    application.handle_event(Event::Key(Key::Char('/')));
    type_query(&mut application, "SIGNAL");
    assert_eq!(
        lines(&application.render())[4].trim_end(),
        "  4 results • Result 1/4"
    );

    application.handle_event(Event::Key(Key::Char('/')));
    assert_eq!(
        lines(&application.render())[3].trim_end(),
        "  Query > SIGNAL"
    );

    application.handle_event(Event::Key(Key::Char('?')));
    assert_eq!(
        lines(&application.render())[0].trim_end(),
        " SONGDIAL / SEARCH EVERYTHING / HELP"
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(
        lines(&application.render())[3].trim_end(),
        "  Query > SIGNAL"
    );

    type_query(&mut application, " Ga");
    assert_eq!(
        lines(&application.render())[4].trim_end(),
        "  1 result • Result 1/1"
    );
    for _ in 0..3 {
        application.handle_event(Event::Key(Key::Backspace));
    }
    assert_eq!(
        lines(&application.render())[4].trim_end(),
        "  4 results • Result 1/4"
    );
}
