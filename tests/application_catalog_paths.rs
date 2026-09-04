use songdial::{
    Application, CatalogId, Effect, Event, Key, PlaybackRequest, PlaybackTarget, Viewport,
};

mod support;

use support::lines;

fn open_radio_stations(application: &mut Application) {
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));
}

fn open_my_playlists(application: &mut Application) {
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));
}

fn expect_load(effect: Effect) -> PlaybackRequest {
    match effect {
        Effect::LoadPlayback(request) => request,
        other => panic!("expected a playback load, got {other:?}"),
    }
}

#[test]
fn radio_stations_lists_the_complete_catalog_and_scrolls_without_losing_its_snapshot() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_radio_stations(&mut application);

    let first_page = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 6, 18, 22, 23].map(|row| first_page[row].trim_end().to_owned()),
        [
            " SONGDIAL / RADIO STATIONS",
            "  RADIO STATIONS",
            "  Continuous music from every Source.",
            "  8 Stations • Station 1/8",
            "  SELECTED > STATION   Night Ledger                         AVAILABLE   [MORROW]",
            "  UNAVAIL !  STATION   Northbound Static                      UNAVAIL   [MORROW]",
            " ↑/k ↓/j move  Enter inspect",
            " p play  n queue  Esc back",
        ]
        .map(str::to_owned)
    );
    assert!(first_page[7].contains("Ambient • AVAILABLE"));
    assert!(first_page[19].contains("Drone • UNAVAIL"));

    for _ in 0..20 {
        application.handle_event(Event::Key(Key::Down));
    }
    let last_page = lines(&application.render());
    assert_eq!(
        (
            last_page[4].trim_end(),
            last_page[6].contains("Daylight Circuit"),
            last_page[6].contains("[HARBOR]"),
            last_page[18].contains("SELECTED > STATION"),
            last_page[18].contains("Open Frequency"),
            last_page[18].contains("[HARBOR]"),
        ),
        ("  8 Stations • Station 8/8", true, true, true, true, true,)
    );
    for station_name in [
        "Night Ledger",
        "Daylight Circuit",
        "Stillwater FM",
        "Kinetic Line",
        "Low Tide Radio",
        "Afterglow Signal",
        "Northbound Static",
        "Open Frequency",
    ] {
        assert!(
            first_page
                .iter()
                .chain(&last_page)
                .any(|line| line.contains(station_name)),
            "missing Station {station_name}"
        );
    }

    let station_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Escape));

    assert_eq!(application.render(), station_snapshot);
}

#[test]
fn unavailable_station_details_remain_inspectable_and_disable_playback_with_the_reason() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_radio_stations(&mut application);
    for _ in 0..6 {
        application.handle_event(Event::Key(Key::Down));
    }
    let browser_snapshot = lines(&application.render())[..20].to_vec();
    assert_eq!(
        lines(&application.render())[23].trim_end(),
        " p unavailable  n queue  Esc back"
    );
    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              ERROR • Cannot play: Signal maintenance is in progress."
    );

    application.handle_event(Event::Key(Key::Enter));

    let details = lines(&application.render());
    assert_eq!(
        [0, 2, 4, 5, 6, 8, 10, 11, 12, 14, 22, 23].map(|row| details[row].trim_end().to_owned()),
        [
            " SONGDIAL / RADIO STATIONS / NORTHBOUND STATIC",
            "  STATION",
            "  Northbound Static",
            "  Drone",
            "  Source  Morrow Audio",
            "  Long-form tonal broadcasts from the northern line.",
            "  Status  UNAVAIL",
            "  Signal maintenance is in progress.",
            "  Enter opened details only. Nothing started playing.",
            "  Queue  Stations are continuous and cannot be added.",
            " p unavailable  n queue  Esc back",
            " ? help  q quit",
        ]
        .map(str::to_owned)
    );
    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              ERROR • Cannot play: Signal maintenance is in progress."
    );

    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(lines(&application.render())[..20], browser_snapshot);
}

#[test]
fn radio_stations_starts_the_shared_live_session_and_keeps_navigation_independent() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_radio_stations(&mut application);

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Station(CatalogId::new("morrow-night-ledger"))
    );
    assert_eq!(
        lines(&application.render())[20].trim_end(),
        " NOW PLAYING  Loading Night Ledger [MORROW]"
    );

    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Escape));

    let home = lines(&application.render());
    assert_eq!(
        [0, 5, 20, 21].map(|row| home[row].trim_end().to_owned()),
        [
            " SONGDIAL / HOME",
            "  > Radio stations                                            SELECTED",
            " NOW PLAYING  Night Ledger [MORROW] • LIVE",
            "              Continuous Station • Queue empty",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn my_playlists_lists_track_counts_and_sources_and_restores_its_snapshot() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_my_playlists(&mut application);

    let playlists = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 6, 7, 14, 15, 22, 23].map(|row| playlists[row].trim_end().to_owned()),
        [
            " SONGDIAL / MY PLAYLISTS",
            "  MY PLAYLISTS",
            "  Personal and saved Playlists from every Source.",
            "  5 Playlists • Playlist 1/5",
            "  SELECTED > PLAYLIST  Deep Work Rotation                   AVAILABLE   [MORROW]",
            "             20 Tracks • AVAILABLE",
            "             PLAYLIST  Empty Room                               EMPTY   [HARBOR]",
            "             0 Tracks • EMPTY",
            " ↑/k ↓/j move  Enter inspect",
            " p play  a add  Esc back",
        ]
        .map(str::to_owned)
    );
    for playlist_name in [
        "Deep Work Rotation",
        "Focus Lines",
        "Calm Current",
        "Energy Shift",
        "Empty Room",
    ] {
        assert!(
            playlists.iter().any(|line| line.contains(playlist_name)),
            "missing Playlist {playlist_name}"
        );
    }

    for _ in 0..10 {
        application.handle_event(Event::Key(Key::Down));
    }
    let empty_playlist_snapshot = application.render();
    let selected_empty = lines(&empty_playlist_snapshot);
    assert!(selected_empty[14].contains("SELECTED > PLAYLIST  Empty Room"));
    assert_eq!(
        selected_empty[23].trim_end(),
        " p unavailable  a unavailable  Esc back"
    );

    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Escape));

    assert_eq!(application.render(), empty_playlist_snapshot);
}

#[test]
fn empty_playlist_details_explain_disabled_playback_and_queue_actions() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_my_playlists(&mut application);
    for _ in 0..4 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));

    let details = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 5, 6, 8, 9, 22, 23].map(|row| details[row].trim_end().to_owned()),
        [
            " SONGDIAL / MY PLAYLISTS / EMPTY ROOM",
            "  PLAYLIST",
            "  Empty Room",
            "  A saved Playlist waiting for its first Track.",
            "  Source  Harbor Sound • 0 Tracks",
            "  TRACKS • EMPTY",
            "  This Playlist has no Tracks in the Demo catalog.",
            "  Esc returns to My playlists without changing Now Playing.",
            " p unavailable  a unavailable  Esc back",
            " n queue  ? help  q quit",
        ]
        .map(str::to_owned)
    );

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              ERROR • Cannot play: Empty Room has no Tracks."
    );

    application.handle_event(Event::Key(Key::Char('a')));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              ERROR • Cannot add: Empty Room has no playable Tracks."
    );
}

#[test]
fn non_empty_playlist_details_reuse_track_browsing_and_restore_my_playlists() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_my_playlists(&mut application);
    let playlists_snapshot = application.render();

    application.handle_event(Event::Key(Key::Enter));
    let details = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 5, 6, 7, 22, 23].map(|row| details[row].trim_end().to_owned()),
        [
            " SONGDIAL / MY PLAYLISTS / DEEP WORK ROTATION",
            "  PLAYLIST",
            "  Deep Work Rotation",
            "  Twenty patient Tracks arranged for a long concentration block.",
            "  Source  Morrow Audio • 20 Tracks",
            "  TRACKS • Track 1/20",
            "  SELECTED > TRACK     Night Geometry                           05:28   [MORROW]",
            " ↑/k ↓/j move  p play",
            " a add  n queue  Esc back",
        ]
        .map(str::to_owned)
    );

    for _ in 0..30 {
        application.handle_event(Event::Key(Key::Down));
    }
    assert!(lines(&application.render())[19].contains("SELECTED > TRACK"));
    assert!(lines(&application.render())[19].contains("Cinder Lines"));

    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), playlists_snapshot);
}

#[test]
fn my_playlists_starts_the_shared_playlist_session_and_preserves_now_playing() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_my_playlists(&mut application);

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Track(CatalogId::new("morrow-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(request.id()));

    let selected_and_playing = lines(&application.render());
    assert!(selected_and_playing[6].contains("SEL+PLAY > PLAYLIST  Deep Work Rotation"));
    assert_eq!(
        [20, 21].map(|row| selected_and_playing[row].trim_end().to_owned()),
        [
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 19 Tracks",
        ]
        .map(str::to_owned)
    );

    application.handle_event(Event::Key(Key::Down));
    let independent = lines(&application.render());
    assert!(independent[6].contains("PLAYING *  PLAYLIST  Deep Work Rotation"));
    assert!(independent[8].contains("SELECTED > PLAYLIST  Focus Lines"));

    application.handle_event(Event::Key(Key::Escape));
    let home = lines(&application.render());
    assert_eq!(
        [0, 6, 20, 21].map(|row| home[row].trim_end().to_owned()),
        [
            " SONGDIAL / HOME",
            "  > My playlists                                              SELECTED",
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 19 Tracks",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn empty_playlist_in_my_playlists_disables_playback_and_queue_add_with_reasons() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_my_playlists(&mut application);
    for _ in 0..4 {
        application.handle_event(Event::Key(Key::Down));
    }

    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              ERROR • Cannot play: Empty Room has no Tracks."
    );

    application.handle_event(Event::Key(Key::Char('a')));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              ERROR • Cannot add: Empty Room has no playable Tracks."
    );
}

#[test]
fn my_playlists_reuses_playlist_queue_expansion_without_interrupting_playback() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_my_playlists(&mut application);
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Key(Key::Char('a')));

    let rendered = lines(&application.render());
    assert_eq!(
        [20, 21, 22].map(|row| rendered[row].trim_end().to_owned()),
        [
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 23 Tracks",
            " QUEUE  Added 4 Tracks from Focus Lines. Queue 23 Tracks",
        ]
        .map(str::to_owned)
    );
}
