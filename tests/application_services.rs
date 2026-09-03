use songdial::{
    Application, CatalogId, Effect, Event, Key, PlaybackRequest, PlaybackTarget, Viewport,
};

mod support;

use support::lines;

fn open_browse_services(application: &mut Application) {
    for _ in 0..3 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
}

fn expect_load(effect: Effect) -> PlaybackRequest {
    match effect {
        Effect::LoadPlayback(request) => request,
        other => panic!("expected a playback load, got {other:?}"),
    }
}

#[test]
fn both_fictional_services_open_source_filtered_shared_catalogs() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_browse_services(&mut application);

    let services = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 6, 7, 8, 9, 22, 23].map(|row| services[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES",
            "  BROWSE SERVICES",
            "  Choose a fictional Service catalog by Source.",
            "  2 Services • Service 1/2",
            "  SELECTED > SERVICE   Morrow Audio                                     [MORROW]",
            "             4 Stations • 2 Playlists • 12 Tracks",
            "             SERVICE   Harbor Sound                                     [HARBOR]",
            "             4 Stations • 3 Playlists • 12 Tracks",
            " ↑/k ↓/j move  Enter open",
            " n queue  Esc back  ? help",
        ]
        .map(str::to_owned)
    );

    application.handle_event(Event::Key(Key::Enter));
    let morrow = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 6, 7, 18, 19].map(|row| morrow[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES / MORROW AUDIO",
            "  MORROW AUDIO",
            "  Source-filtered Stations, Playlists, and Tracks.",
            "  4 Stations • 2 Playlists • 12 Tracks • Item 1/18",
            "  SELECTED > STATION   Night Ledger                                     [MORROW]",
            "             Ambient • AVAILABLE",
            "             TRACK     Night Geometry                                   [MORROW]",
            "             Sable Circuit • AVAILABLE",
        ]
        .map(str::to_owned)
    );

    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));
    let harbor = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 4, 6, 7].map(|row| harbor[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES / HARBOR SOUND",
            "  HARBOR SOUND",
            "  Source-filtered Stations, Playlists, and Tracks.",
            "  4 Stations • 3 Playlists • 12 Tracks • Item 1/19",
            "  SELECTED > STATION   Daylight Circuit                                 [HARBOR]",
            "             Minimal electronic • LOADING",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn service_content_details_open_without_playback_and_restore_each_catalog_snapshot() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_browse_services(&mut application);
    application.handle_event(Event::Key(Key::Enter));

    let station_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    let station = lines(&application.render());
    assert_eq!(
        [0, 2, 4, 5, 6, 8, 10, 12].map(|row| station[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES / MORROW AUDIO / NIGHT LEDGER",
            "  STATION",
            "  Night Ledger",
            "  Ambient",
            "  Source  Morrow Audio",
            "  Unhurried ambient transmissions for sustained attention.",
            "  Status  Available",
            "  Enter opened details only. Nothing started playing.",
        ]
        .map(str::to_owned)
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), station_snapshot);

    for _ in 0..4 {
        application.handle_event(Event::Key(Key::Down));
    }
    let playlist_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    let playlist = lines(&application.render());
    assert_eq!(
        [0, 2, 3, 5, 6, 7].map(|row| playlist[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES / MORROW AUDIO / DEEP WORK ROTATION",
            "  PLAYLIST",
            "  Deep Work Rotation",
            "  Source  Morrow Audio • 20 Tracks",
            "  TRACKS • Track 1/20",
            "  SELECTED > TRACK   Night Geometry              Sable Circuit          [MORROW]",
        ]
        .map(str::to_owned)
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), playlist_snapshot);

    for _ in 0..2 {
        application.handle_event(Event::Key(Key::Down));
    }
    let track_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    let track = lines(&application.render());
    assert_eq!(
        [0, 2, 4, 5, 6, 8, 10, 12, 22, 23].map(|row| track[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES / MORROW AUDIO / NIGHT GEOMETRY",
            "  TRACK",
            "  Night Geometry",
            "  Sable Circuit",
            "  Source  Morrow Audio",
            "  Duration  05:28",
            "  Status  Available",
            "  Enter opened details only. Nothing started playing.",
            " p play  Space pause  n queue",
            " a add  Esc back  ? help",
        ]
        .map(str::to_owned)
    );
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), track_snapshot);
}

#[test]
fn service_catalog_content_reuses_playback_and_keeps_duplicate_tracks_source_scoped() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_browse_services(&mut application);
    application.handle_event(Event::Key(Key::Enter));

    let station_request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        station_request.target(),
        &PlaybackTarget::Station(CatalogId::new("morrow-night-ledger"))
    );
    application.handle_event(Event::PlaybackLoaded(station_request.id()));
    assert_eq!(
        lines(&application.render())[20].trim_end(),
        " NOW PLAYING  Night Ledger [MORROW] • LIVE"
    );

    for _ in 0..4 {
        application.handle_event(Event::Key(Key::Down));
    }
    let playlist_request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        playlist_request.target(),
        &PlaybackTarget::Track(CatalogId::new("morrow-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(playlist_request.id()));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Queue 19 Tracks"
    );

    for _ in 0..2 {
        application.handle_event(Event::Key(Key::Down));
    }
    let morrow_duplicate = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        morrow_duplicate.target(),
        &PlaybackTarget::Track(CatalogId::new("morrow-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(morrow_duplicate.id()));
    assert_eq!(
        lines(&application.render())[20].trim_end(),
        " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28"
    );

    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..7 {
        application.handle_event(Event::Key(Key::Down));
    }
    let harbor_duplicate = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        harbor_duplicate.target(),
        &PlaybackTarget::Track(CatalogId::new("harbor-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(harbor_duplicate.id()));

    let harbor = lines(&application.render());
    assert!(harbor[18].contains("SEL+PLAY > TRACK"));
    assert!(harbor[18].contains("Night Geometry"));
    assert!(harbor[18].contains("[HARBOR]"));
    assert_eq!(
        [20, 21].map(|row| harbor[row].trim_end().to_owned()),
        [
            " NOW PLAYING  Night Geometry [HARBOR] • PLAYING • 00:00/05:31",
            "              Queue 0 Tracks",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn service_catalog_reuses_queue_feedback_for_every_content_type() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_browse_services(&mut application);
    application.handle_event(Event::Key(Key::Enter));

    application.handle_event(Event::Key(Key::Char('a')));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Cannot add: Stations are continuous and cannot be queued."
    );

    for _ in 0..6 {
        application.handle_event(Event::Key(Key::Down));
    }
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Char('a')));
    let track_added = lines(&application.render());
    assert_eq!(
        [20, 21, 22].map(|row| track_added[row].trim_end().to_owned()),
        [
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 1 Track",
            " QUEUE  Added Stillwater Signal to Queue. Queue 1 Track",
        ]
        .map(str::to_owned)
    );

    for _ in 0..3 {
        application.handle_event(Event::Key(Key::Up));
    }
    application.handle_event(Event::Key(Key::Char('a')));
    let playlist_added = lines(&application.render());
    assert_eq!(
        [20, 21, 22].map(|row| playlist_added[row].trim_end().to_owned()),
        [
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 20 Tracks",
            " QUEUE  Added 19 Tracks from Deep Work Rotation. Queue 20 Tracks",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn back_restores_service_catalog_details_and_scrolled_now_playing_snapshots_exactly() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_browse_services(&mut application);
    let services_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));

    for _ in 0..4 {
        application.handle_event(Event::Key(Key::Down));
    }
    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    for _ in 0..30 {
        application.handle_event(Event::Key(Key::Down));
    }
    let service_snapshot = application.render();
    let service = lines(&service_snapshot);
    assert_eq!(
        service[4].trim_end(),
        "  4 Stations • 2 Playlists • 12 Tracks • Item 18/18"
    );
    assert!(service[18].contains("SELECTED > TRACK"));
    assert!(service[18].contains("Long Form"));
    assert!(service[18].contains("[MORROW]"));

    application.handle_event(Event::Key(Key::Enter));
    let details_snapshot = application.render();
    assert_eq!(
        lines(&details_snapshot)[0].trim_end(),
        " SONGDIAL / BROWSE SERVICES / MORROW AUDIO / LONG FORM"
    );

    application.handle_event(Event::Key(Key::Char('n')));
    for _ in 0..30 {
        application.handle_event(Event::Key(Key::Down));
    }
    let queue_snapshot = application.render();
    let queue = lines(&queue_snapshot);
    assert_eq!(queue[5].trim_end(), "  QUEUE • 19 Tracks • Track 19/19");
    assert!(queue[19].contains("SELECTED > TRACK"));
    assert!(queue[19].contains("Cinder Lines"));

    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), queue_snapshot);

    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), details_snapshot);
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), service_snapshot);
    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(
        &lines(&application.render())[..20],
        &lines(&services_snapshot)[..20]
    );

    application.handle_event(Event::Key(Key::Escape));
    let home = lines(&application.render());
    assert_eq!(home[0].trim_end(), " SONGDIAL / HOME");
    assert_eq!(
        home[7].trim_end(),
        "  > Browse services                                           SELECTED"
    );
}

#[test]
fn service_track_details_reuse_playback_and_queue_actions() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_browse_services(&mut application);
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..6 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));

    let request = expect_load(application.handle_event(Event::Key(Key::Char('p'))));
    assert_eq!(
        request.target(),
        &PlaybackTarget::Track(CatalogId::new("morrow-night-geometry"))
    );
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Char('a')));

    let details = lines(&application.render());
    assert_eq!(
        [0, 20, 21, 22].map(|row| details[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES / MORROW AUDIO / NIGHT GEOMETRY",
            " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            "              Queue 1 Track",
            " QUEUE  Added Night Geometry to Queue. Queue 1 Track",
        ]
        .map(str::to_owned)
    );
}

#[test]
fn service_catalog_preserves_empty_and_unavailable_action_feedback() {
    let mut application = Application::new(Viewport::new(80, 24));
    open_browse_services(&mut application);
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));

    for _ in 0..6 {
        application.handle_event(Event::Key(Key::Down));
    }
    assert_eq!(
        lines(&application.render())[23].trim_end(),
        " p unavailable  a unavailable  Esc back"
    );
    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Cannot play: Empty Room has no Tracks."
    );
    application.handle_event(Event::Key(Key::Char('a')));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Cannot add: Empty Room has no playable Tracks."
    );

    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    let unavailable = lines(&application.render());
    assert!(unavailable[18].contains("SEL+UNAV > TRACK"));
    assert!(unavailable[18].contains("Blueprint Sky"));
    assert!(unavailable[18].contains("[HARBOR]"));
    assert_eq!(
        unavailable[23].trim_end(),
        " p unavailable  a unavailable  Esc back"
    );

    application.handle_event(Event::Key(Key::Enter));
    let details = lines(&application.render());
    assert_eq!(
        [0, 10, 11, 22, 23].map(|row| details[row].trim_end().to_owned()),
        [
            " SONGDIAL / BROWSE SERVICES / HARBOR SOUND / BLUEPRINT SKY",
            "  Status  Unavailable",
            "  This recording is unavailable in the Demo catalog.",
            " p unavailable  Space pause  n queue",
            " a unavailable  Esc back  ? help",
        ]
        .map(str::to_owned)
    );
    assert_eq!(
        application.handle_event(Event::Key(Key::Char('p'))),
        Effect::None
    );
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Cannot play: This recording is unavailable in the Demo catalog."
    );
    application.handle_event(Event::Key(Key::Char('a')));
    assert_eq!(
        lines(&application.render())[21].trim_end(),
        "              Cannot add: This recording is unavailable in the Demo catalog."
    );
}
