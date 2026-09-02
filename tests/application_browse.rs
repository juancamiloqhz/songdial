use ratatui::style::Color;
use songdial::{Application, DemoCatalog, Event, Key, ListeningIntent, Service, Station, Viewport};

mod support;

use support::lines;

#[test]
fn home_opens_the_ordered_listening_intents_without_starting_playback() {
    let mut application = Application::new(Viewport::new(80, 24));

    application.handle_event(Event::Key(Key::Enter));

    let rendered = lines(&application.render());
    let excerpt =
        [0, 2, 3, 5, 6, 7, 8, 9, 10, 20, 21, 22, 23].map(|row| rendered[row].trim_end().to_owned());
    assert_eq!(
        excerpt,
        [
            " SONGDIAL / MOOD & ACTIVITY",
            "  MOOD & ACTIVITY",
            "  Choose what fits right now.",
            "  > Deep Work                                                 SELECTED",
            "    Focus",
            "    Flow",
            "    Calm",
            "    Energy",
            "    Reset",
            " NOW PLAYING  Nothing playing",
            "              Open a choice to keep exploring.",
            " ↑/k up  ↓/j down  Enter open",
            " Esc back  ? help  q quit",
        ]
    );
}

#[test]
fn listening_intent_scrolls_through_matching_stations_and_playlists_with_sources() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));

    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }

    let rendered = lines(&application.render());
    assert_eq!(
        (
            rendered[0].trim_end(),
            rendered[2].trim_end(),
            rendered[3].trim_end(),
            rendered[4].trim_end(),
            rendered[6].contains("STATION"),
            rendered[6].contains("[MORROW]"),
            rendered[18].contains("SELECTED > PLAYLIST"),
            rendered[18].contains("[MORROW]"),
            rendered[20].trim_end(),
        ),
        (
            " SONGDIAL / MOOD & ACTIVITY / DEEP WORK",
            "  DEEP WORK",
            "  Steady, low-distraction sound for sustained concentration.",
            "  8 Stations • 5 Playlists • Choice 9/13",
            true,
            true,
            true,
            true,
            " NOW PLAYING  Nothing playing",
        )
    );
}

#[test]
fn selected_listening_intent_match_highlights_its_complete_source_badge() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));

    let buffer = application.render();
    let rendered = lines(&buffer);
    let source_start = rendered[6].find("[MORROW]").expect("selected Source badge");
    let source_backgrounds = (source_start..source_start + "[MORROW]".len())
        .map(|column| buffer[(u16::try_from(column).expect("terminal column"), 6)].bg)
        .collect::<Vec<_>>();

    assert_eq!(
        source_backgrounds,
        vec![Color::Rgb(214, 166, 75); "[MORROW]".len()]
    );
}

#[test]
fn selected_playlist_track_highlights_its_complete_source_badge() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));

    let buffer = application.render();
    let rendered = lines(&buffer);
    let source_start = rendered[7].find("[MORROW]").expect("selected Source badge");
    let source_backgrounds = (source_start..source_start + "[MORROW]".len())
        .map(|column| buffer[(u16::try_from(column).expect("terminal column"), 7)].bg)
        .collect::<Vec<_>>();

    assert_eq!(
        source_backgrounds,
        vec![Color::Rgb(214, 166, 75); "[MORROW]".len()]
    );
}

#[test]
fn mood_and_activity_help_uses_listener_facing_language() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Char('?')));
    let chooser_help = lines(&application.render());

    application.handle_event(Event::Key(Key::Char('?')));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Char('?')));
    let direction_help = lines(&application.render());

    assert_eq!(
        (chooser_help[12].trim_end(), direction_help[12].trim_end(),),
        (
            "  Mood & activity: choose what fits with Enter.",
            "  Open a Station or Playlist with Enter.",
        )
    );
}

#[test]
fn station_details_open_without_playback_and_back_restores_the_exact_snapshot() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..7 {
        application.handle_event(Event::Key(Key::Down));
    }
    let intent_snapshot = application.render();

    application.handle_event(Event::Key(Key::Enter));

    let details = lines(&application.render());
    let excerpt =
        [0, 2, 4, 5, 6, 8, 10, 12, 20, 21, 22].map(|row| details[row].trim_end().to_owned());
    assert_eq!(
        excerpt,
        [
            " SONGDIAL / MOOD & ACTIVITY / DEEP WORK / OPEN FREQUENCY",
            "  STATION",
            "  Open Frequency",
            "  Indie instrumental",
            "  Source  Harbor Sound",
            "  Guitar-led instrumentals with a steady horizon.",
            "  Status  Available",
            "  Enter opened details only. Nothing started playing.",
            " NOW PLAYING  Nothing playing",
            "              Open a choice to keep exploring.",
            " Esc back  ? help  q quit",
        ]
    );

    application.handle_event(Event::Key(Key::Escape));

    assert_eq!(application.render(), intent_snapshot);
}

#[test]
fn long_playlist_scrolls_to_both_boundaries_and_restores_its_parent_snapshot() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    for _ in 0..8 {
        application.handle_event(Event::Key(Key::Down));
    }
    let intent_snapshot = application.render();
    application.handle_event(Event::Key(Key::Enter));
    let first_track = application.render();

    application.handle_event(Event::Key(Key::Up));
    application.handle_event(Event::Key(Key::Char('k')));
    assert_eq!(application.render(), first_track);

    for _ in 0..30 {
        application.handle_event(Event::Key(Key::Down));
    }
    let last_track = lines(&application.render());
    assert_eq!(
        (
            last_track[0].trim_end(),
            last_track[2].trim_end(),
            last_track[3].trim_end(),
            last_track[5].trim_end(),
            last_track[6].trim_end(),
            last_track[19].contains("SELECTED > TRACK"),
            last_track[19].contains("Cinder Lines"),
            last_track[19].contains("[HARBOR]"),
            last_track[20].trim_end(),
        ),
        (
            " SONGDIAL / MOOD & ACTIVITY / DEEP WORK / DEEP WORK ROTATION",
            "  PLAYLIST",
            "  Deep Work Rotation",
            "  Source  Morrow Audio • 20 Tracks",
            "  TRACKS • Track 20/20",
            true,
            true,
            true,
            " NOW PLAYING  Nothing playing",
        )
    );

    let last_boundary = application.render();
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Char('j')));
    assert_eq!(application.render(), last_boundary);

    application.handle_event(Event::Key(Key::Escape));
    assert_eq!(application.render(), intent_snapshot);
}

#[test]
fn empty_intent_and_playlist_explain_their_deterministic_next_action() {
    let mut empty_intent = Application::new(Viewport::new(80, 24));
    empty_intent.handle_event(Event::Key(Key::Enter));
    for _ in 0..5 {
        empty_intent.handle_event(Event::Key(Key::Down));
    }
    empty_intent.handle_event(Event::Key(Key::Enter));
    let intent = lines(&empty_intent.render());

    let mut empty_playlist = Application::new(Viewport::new(80, 24));
    empty_playlist.handle_event(Event::Key(Key::Enter));
    empty_playlist.handle_event(Event::Key(Key::Enter));
    for _ in 0..12 {
        empty_playlist.handle_event(Event::Key(Key::Down));
    }
    empty_playlist.handle_event(Event::Key(Key::Enter));
    let playlist = lines(&empty_playlist.render());

    assert_eq!(
        (
            [0, 4, 7, 8, 20].map(|row| intent[row].trim_end().to_owned()),
            [0, 5, 6, 8, 9, 20].map(|row| playlist[row].trim_end().to_owned()),
        ),
        (
            [
                " SONGDIAL / MOOD & ACTIVITY / RESET",
                "  0 Stations • 0 Playlists",
                "  Nothing matches Reset in the Demo catalog.",
                "  Esc returns to Mood & activity to choose another direction.",
                " NOW PLAYING  Nothing playing",
            ]
            .map(str::to_owned),
            [
                " SONGDIAL / MOOD & ACTIVITY / DEEP WORK / EMPTY ROOM",
                "  Source  Harbor Sound • 0 Tracks",
                "  TRACKS • Empty",
                "  This Playlist has no Tracks in the Demo catalog.",
                "  Esc returns to Deep Work without changing Now Playing.",
                " NOW PLAYING  Nothing playing",
            ]
            .map(str::to_owned),
        )
    );
}

#[test]
fn application_accepts_dynamically_owned_replacement_catalog_data() {
    let service_id = String::from("local-service");
    let station_id = String::from("local-reading-room");
    let intent_name = String::from("Close Reading");
    let catalog = DemoCatalog::new(
        vec![Service::new(
            service_id.clone(),
            String::from("Local Service"),
            String::from("LOCAL"),
        )],
        vec![ListeningIntent::new(
            String::from("close-reading"),
            intent_name.clone(),
            String::from("Quiet detail for a demanding text."),
            &[station_id.as_str()],
            &[],
        )],
        vec![Station::available(
            station_id,
            String::from("Reading Room"),
            String::from("Minimal piano"),
            String::from("A restrained signal for careful reading."),
            service_id,
        )],
        vec![],
        vec![],
    );
    let mut application = Application::with_catalog(Viewport::new(80, 24), catalog);

    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));

    let rendered = lines(&application.render());
    assert_eq!(
        (
            rendered[0].trim_end(),
            rendered[2].trim_end(),
            rendered[3].trim_end(),
            rendered[6].contains("Reading Room"),
            rendered[6].contains("[LOCAL]"),
        ),
        (
            " SONGDIAL / MOOD & ACTIVITY / CLOSE READING",
            "  CLOSE READING",
            "  Quiet detail for a demanding text.",
            true,
            true,
        )
    );
}
