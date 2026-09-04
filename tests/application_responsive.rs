use ratatui::style::{Color, Modifier};
use songdial::{
    Application, ApplicationOptions, DemoCatalog, Effect, Event, Key, Playlist, Service, Station,
    Track, Viewport,
};

mod support;

use support::lines;

fn assert_exact_text_frame(application: &Application, default_row: &str, rows: &[(usize, &str)]) {
    let actual = lines(&application.render())
        .into_iter()
        .map(|line| line.trim_end().to_owned())
        .collect::<Vec<_>>();
    let mut expected = vec![default_row.to_owned(); actual.len()];
    for &(row, text) in rows {
        expected[row] = text.to_owned();
    }
    assert_eq!(actual, expected);
}

fn playback_request(effect: Effect) -> songdial::PlaybackRequest {
    let Effect::LoadPlayback(request) = effect else {
        panic!("expected playback loading effect, got {effect:?}");
    };
    request
}

#[test]
fn exact_buffer_matrix_covers_terminal_sizes_options_content_and_states() {
    let compact = Application::new(Viewport::new(80, 24));
    assert_exact_text_frame(
        &compact,
        "",
        &[
            (0, " SONGDIAL / HOME"),
            (2, "  CHOOSE WHAT FITS RIGHT NOW"),
            (
                4,
                "  > Mood & activity                                           SELECTED",
            ),
            (5, "    Radio stations"),
            (6, "    My playlists"),
            (7, "    Browse services"),
            (8, "    Search everything"),
            (
                10,
                "  Enter opens a Destination. Playback always starts separately.",
            ),
            (20, " NOW PLAYING  Nothing playing"),
            (21, "              Open a choice to keep exploring."),
            (22, " ↑/k up  ↓/j down  Enter open"),
            (23, " n queue  ? help  q quit"),
        ],
    );

    let intermediate = Application::new(Viewport::new(100, 30));
    assert_exact_text_frame(
        &intermediate,
        "",
        &[
            (0, " SONGDIAL / HOME"),
            (2, "  CHOOSE WHAT FITS RIGHT NOW"),
            (
                4,
                "  > Mood & activity                                           SELECTED",
            ),
            (5, "    Radio stations"),
            (6, "    My playlists"),
            (7, "    Browse services"),
            (8, "    Search everything"),
            (
                10,
                "  Enter opens a Destination. Playback always starts separately.",
            ),
            (26, " NOW PLAYING  Nothing playing"),
            (27, "              Open a choice to keep exploring."),
            (28, " ↑/k up  ↓/j down  Enter open"),
            (29, " n queue  ? help  q quit"),
        ],
    );

    let wide = Application::new(Viewport::new(120, 40));
    assert_exact_text_frame(
        &wide,
        "                                                          │",
        &[
            (0, " SONGDIAL / HOME"),
            (
                2,
                "  CHOOSE WHAT FITS RIGHT NOW                              │ DETAIL LENS / READ ONLY",
            ),
            (
                4,
                "  > Mood & activity                                       │ DESTINATION",
            ),
            (
                5,
                "    Radio stations                                        │ Mood & activity",
            ),
            (
                6,
                "    My playlists                                          │",
            ),
            (
                7,
                "    Browse services                                       │ Start from a Listening intent before choosing a Source.",
            ),
            (
                8,
                "    Search everything                                     │",
            ),
            (
                9,
                "                                                          │ Enter still opens the same Destination.",
            ),
            (
                10,
                "  Enter opens a Destination. Playback always starts separa│",
            ),
            (36, " NOW PLAYING  Nothing playing"),
            (37, "              Open a choice to keep exploring."),
            (38, " ↑/k up  ↓/j down  Enter open"),
            (39, " n queue  ? help  q quit"),
        ],
    );

    let guard = Application::new(Viewport::new(79, 23));
    assert_exact_text_frame(
        &guard,
        "",
        &[
            (8, "                           SONGDIAL NEEDS MORE ROOM"),
            (10, "                             Current  79×23 cells"),
            (11, "                             Required 80×24 cells"),
            (
                13,
                "                   Resize to recover the unchanged session.",
            ),
            (14, "                                    q quit"),
        ],
    );

    let mut no_color =
        Application::with_options(Viewport::new(80, 24), ApplicationOptions::new(false, true));
    no_color.handle_event(Event::Key(Key::Down));
    assert_exact_text_frame(
        &no_color,
        "",
        &[
            (0, " SONGDIAL / HOME"),
            (2, "  CHOOSE WHAT FITS RIGHT NOW"),
            (4, "    Mood & activity"),
            (
                5,
                "  > Radio stations                                            SELECTED",
            ),
            (6, "    My playlists"),
            (7, "    Browse services"),
            (8, "    Search everything"),
            (
                10,
                "  Enter opens a Destination. Playback always starts separately.",
            ),
            (20, " NOW PLAYING  Nothing playing"),
            (21, "              Open a choice to keep exploring."),
            (22, " ↑/k up  ↓/j down  Enter open"),
            (23, " n queue  ? help  q quit"),
        ],
    );

    let mut loading =
        Application::with_options(Viewport::new(80, 24), ApplicationOptions::new(true, false));
    loading.handle_event(Event::Key(Key::Down));
    loading.handle_event(Event::Key(Key::Enter));
    let _ = playback_request(loading.handle_event(Event::Key(Key::Char('p'))));
    assert_exact_text_frame(
        &loading,
        "",
        &[
            (0, " SONGDIAL / RADIO STATIONS"),
            (2, "  RADIO STATIONS"),
            (3, "  Continuous music from every Source."),
            (4, "  8 Stations • Station 1/8"),
            (
                6,
                "  SELECTED > STATION   Night Ledger                         AVAILABLE   [MORROW]",
            ),
            (7, "             Ambient • AVAILABLE"),
            (
                8,
                "  LOADING ~  STATION   Daylight Circuit                       LOADING   [HARBOR]",
            ),
            (9, "             Minimal electronic • LOADING"),
            (
                10,
                "             STATION   Stillwater FM                        AVAILABLE   [MORROW]",
            ),
            (11, "             Ambient piano • AVAILABLE"),
            (
                12,
                "             STATION   Kinetic Line                         AVAILABLE   [HARBOR]",
            ),
            (13, "             Instrumental pulse • AVAILABLE"),
            (
                14,
                "             STATION   Low Tide Radio                       AVAILABLE   [MORROW]",
            ),
            (15, "             Downtempo • AVAILABLE"),
            (
                16,
                "             STATION   Afterglow Signal                     AVAILABLE   [HARBOR]",
            ),
            (17, "             Warm electronica • AVAILABLE"),
            (
                18,
                "  UNAVAIL !  STATION   Northbound Static                      UNAVAIL   [MORROW]",
            ),
            (19, "             Drone • UNAVAIL"),
            (20, " NOW PLAYING  Loading Night Ledger [MORROW]"),
            (21, "              Waiting for simulated playback."),
            (22, " ↑/k ↓/j move  Enter inspect"),
            (23, " p play  n queue  Esc back"),
        ],
    );

    let mut unicode = Application::with_catalog(Viewport::new(80, 24), unicode_catalog());
    open_only_service(&mut unicode);
    assert_exact_text_frame(
        &unicode,
        "",
        &[
            (0, " SONGDIAL / BROWSE SERVICES / מקור 音 源"),
            (2, "  מקור 音 源"),
            (3, "  Source-filtered Stations, Playlists, and Tracks."),
            (4, "  1 Stations • 0 Playlists • 0 Tracks • Item 1/1"),
            (
                6,
                "  SELECTED > STATION   深 い 集 中 の た め の 非 常 に 長 い 放 送  e\u{301}lan…   LOADING     [源 泉 ]",
            ),
            (7, "             静 か な 電 子 音 楽  • LOADING"),
            (20, " NOW PLAYING  Nothing playing"),
            (21, "              Open a choice to keep exploring."),
            (22, " ↑/k ↓/j move  Enter inspect"),
            (23, " p play  n queue  Esc back"),
        ],
    );

    let mut empty = Application::with_catalog(Viewport::new(80, 24), empty_catalog());
    empty.handle_event(Event::Key(Key::Down));
    empty.handle_event(Event::Key(Key::Enter));
    assert_exact_text_frame(
        &empty,
        "",
        &[
            (0, " SONGDIAL / RADIO STATIONS"),
            (2, "  RADIO STATIONS"),
            (3, "  Continuous music from every Source."),
            (4, "  0 Stations • EMPTY"),
            (7, "  No Stations are available."),
            (8, "  Esc returns Home to choose another path."),
            (20, " NOW PLAYING  Nothing playing"),
            (21, "              Open a choice to keep exploring."),
            (22, " n queue  Esc back  ? help  q quit"),
        ],
    );

    let mut unavailable = Application::new(Viewport::new(80, 24));
    unavailable.handle_event(Event::Key(Key::Down));
    unavailable.handle_event(Event::Key(Key::Enter));
    for _ in 0..6 {
        unavailable.handle_event(Event::Key(Key::Down));
    }
    assert_exact_text_frame(
        &unavailable,
        "",
        &[
            (0, " SONGDIAL / RADIO STATIONS"),
            (2, "  RADIO STATIONS"),
            (3, "  Continuous music from every Source."),
            (4, "  8 Stations • Station 7/8"),
            (
                6,
                "             STATION   Night Ledger                         AVAILABLE   [MORROW]",
            ),
            (7, "             Ambient • AVAILABLE"),
            (
                8,
                "  LOADING ~  STATION   Daylight Circuit                       LOADING   [HARBOR]",
            ),
            (9, "             Minimal electronic • LOADING"),
            (
                10,
                "             STATION   Stillwater FM                        AVAILABLE   [MORROW]",
            ),
            (11, "             Ambient piano • AVAILABLE"),
            (
                12,
                "             STATION   Kinetic Line                         AVAILABLE   [HARBOR]",
            ),
            (13, "             Instrumental pulse • AVAILABLE"),
            (
                14,
                "             STATION   Low Tide Radio                       AVAILABLE   [MORROW]",
            ),
            (15, "             Downtempo • AVAILABLE"),
            (
                16,
                "             STATION   Afterglow Signal                     AVAILABLE   [HARBOR]",
            ),
            (17, "             Warm electronica • AVAILABLE"),
            (
                18,
                "  SEL+UNAV > STATION   Northbound Static                      UNAVAIL   [MORROW]",
            ),
            (19, "             Drone • UNAVAIL"),
            (20, " NOW PLAYING  Nothing playing"),
            (21, "              Open a choice to keep exploring."),
            (22, " ↑/k ↓/j move  Enter inspect"),
            (23, " p unavailable  n queue  Esc back"),
        ],
    );

    let mut error = Application::new(Viewport::new(80, 24));
    error.handle_event(Event::Key(Key::Down));
    error.handle_event(Event::Key(Key::Enter));
    let request = playback_request(error.handle_event(Event::Key(Key::Char('p'))));
    error.handle_event(Event::PlaybackFailed {
        request_id: request.id(),
        reason: "The simulated signal could not load.".to_owned(),
    });
    assert_exact_text_frame(
        &error,
        "",
        &[
            (0, " SONGDIAL / RADIO STATIONS"),
            (2, "  RADIO STATIONS"),
            (3, "  Continuous music from every Source."),
            (4, "  8 Stations • Station 1/8"),
            (
                6,
                "  SELECTED > STATION   Night Ledger                         AVAILABLE   [MORROW]",
            ),
            (7, "             Ambient • AVAILABLE"),
            (
                8,
                "  LOADING ~  STATION   Daylight Circuit                       LOADING   [HARBOR]",
            ),
            (9, "             Minimal electronic • LOADING"),
            (
                10,
                "             STATION   Stillwater FM                        AVAILABLE   [MORROW]",
            ),
            (11, "             Ambient piano • AVAILABLE"),
            (
                12,
                "             STATION   Kinetic Line                         AVAILABLE   [HARBOR]",
            ),
            (13, "             Instrumental pulse • AVAILABLE"),
            (
                14,
                "             STATION   Low Tide Radio                       AVAILABLE   [MORROW]",
            ),
            (15, "             Downtempo • AVAILABLE"),
            (
                16,
                "             STATION   Afterglow Signal                     AVAILABLE   [HARBOR]",
            ),
            (17, "             Warm electronica • AVAILABLE"),
            (
                18,
                "  UNAVAIL !  STATION   Northbound Static                      UNAVAIL   [MORROW]",
            ),
            (19, "             Drone • UNAVAIL"),
            (20, " NOW PLAYING  Nothing playing"),
            (
                21,
                "              ERROR • Playback failed: The simulated signal could not load.",
            ),
            (22, " ↑/k ↓/j move  Enter inspect"),
            (23, " p play  n queue  Esc back"),
        ],
    );

    let stopped_catalog = DemoCatalog::new(
        vec![Service::new("source", "Local Source", "LOCAL")],
        vec![],
        vec![],
        vec![Playlist::new(
            "playlist",
            "Brief Playlist",
            "One brief Track.",
            "source",
            &["brief"],
        )],
        vec![Track::new(
            "brief",
            "Brief Signal",
            "Test Tone",
            "source",
            1,
        )],
    );
    let mut stopped = Application::with_catalog(Viewport::new(80, 24), stopped_catalog);
    for _ in 0..2 {
        stopped.handle_event(Event::Key(Key::Down));
    }
    stopped.handle_event(Event::Key(Key::Enter));
    let request = playback_request(stopped.handle_event(Event::Key(Key::Char('p'))));
    stopped.handle_event(Event::PlaybackLoaded(request.id()));
    stopped.handle_event(Event::Tick);
    assert_exact_text_frame(
        &stopped,
        "",
        &[
            (0, " SONGDIAL / MY PLAYLISTS"),
            (2, "  MY PLAYLISTS"),
            (3, "  Personal and saved Playlists from every Source."),
            (4, "  1 Playlists • Playlist 1/1"),
            (
                6,
                "  SELECTED > PLAYLIST  Brief Playlist                       AVAILABLE    [LOCAL]",
            ),
            (7, "             1 Track • AVAILABLE"),
            (
                20,
                " NOW PLAYING  Brief Signal [LOCAL] • STOPPED • 00:01/00:01",
            ),
            (21, "              Queue 0 Tracks"),
            (22, " ↑/k ↓/j move  Enter inspect"),
            (23, " p play  a add  Esc back"),
        ],
    );

    let mut long_queue = Application::new(Viewport::new(80, 24));
    for _ in 0..2 {
        long_queue.handle_event(Event::Key(Key::Down));
    }
    long_queue.handle_event(Event::Key(Key::Enter));
    let request = playback_request(long_queue.handle_event(Event::Key(Key::Char('p'))));
    long_queue.handle_event(Event::PlaybackLoaded(request.id()));
    long_queue.handle_event(Event::Key(Key::Char('n')));
    assert_exact_text_frame(
        &long_queue,
        "",
        &[
            (0, " SONGDIAL / NOW PLAYING"),
            (2, "  NOW PLAYING & QUEUE"),
            (
                3,
                "  CURRENT  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            ),
            (5, "  QUEUE • 19 Tracks • Track 1/19"),
            (
                6,
                "  SELECTED > TRACK     Night Geometry                           05:31   [HARBOR]",
            ),
            (
                7,
                "             TRACK     Stillwater Signal                        04:44   [MORROW]",
            ),
            (
                8,
                "             TRACK     Stillwater Signal                        04:46   [HARBOR]",
            ),
            (
                9,
                "             TRACK     Slow Aperture                            05:02   [MORROW]",
            ),
            (
                10,
                "             TRACK     Soft Machines                            04:36   [HARBOR]",
            ),
            (
                11,
                "             TRACK     Quiet Index                              05:15   [MORROW]",
            ),
            (
                12,
                "             TRACK     Glass Hours                              04:07   [HARBOR]",
            ),
            (
                13,
                "             TRACK     Copper Rain                              04:53   [MORROW]",
            ),
            (
                14,
                "             TRACK     Parallel Dawn                            04:21   [HARBOR]",
            ),
            (
                15,
                "             TRACK     Signal Garden                            05:08   [MORROW]",
            ),
            (
                16,
                "             TRACK     Warm Circuit                             04:15   [HARBOR]",
            ),
            (
                17,
                "             TRACK     Northern Room                            05:22   [MORROW]",
            ),
            (
                18,
                "             TRACK     Paper Satellites                         04:29   [HARBOR]",
            ),
            (
                19,
                "             TRACK     Low Light Method                         04:57   [MORROW]",
            ),
            (
                20,
                " NOW PLAYING  Night Geometry [MORROW] • PLAYING • 00:00/05:28",
            ),
            (21, "              Queue 19 Tracks"),
            (22, " ↑/k ↓/j move  Enter inspect"),
            (23, " p play  d remove  Esc back"),
        ],
    );
}

#[test]
fn minimum_size_guard_is_recoverable_and_freezes_navigation() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Down));
    let compact = application.render();

    application.handle_event(Event::Resize(Viewport::new(79, 23)));
    for key in [Key::Down, Key::Enter, Key::Char('/'), Key::Escape] {
        assert_eq!(application.handle_event(Event::Key(key)), Effect::None);
    }
    let guard = lines(&application.render());

    assert_eq!(
        [8, 10, 11, 13, 14].map(|row| guard[row].trim_end().to_owned()),
        [
            "                           SONGDIAL NEEDS MORE ROOM",
            "                             Current  79×23 cells",
            "                             Required 80×24 cells",
            "                   Resize to recover the unchanged session.",
            "                                    q quit",
        ]
        .map(str::to_owned)
    );

    application.handle_event(Event::Resize(Viewport::new(80, 24)));
    assert_eq!(application.render(), compact);
}

#[test]
fn minimum_size_guard_keeps_both_quit_paths_available() {
    for quit_key in [Key::Char('q'), Key::CtrlC] {
        let mut application = Application::new(Viewport::new(1, 1));

        assert_eq!(application.handle_event(Event::Key(quit_key)), Effect::Quit);
        assert_eq!(
            application.render().area,
            ratatui::layout::Rect::new(0, 0, 1, 1)
        );
    }
}

#[test]
fn wide_home_adds_a_read_only_detail_lens_from_usable_content_width() {
    let mut application = Application::new(Viewport::new(100, 30));
    let intermediate = lines(&application.render());
    assert!(
        intermediate
            .iter()
            .all(|line| !line.contains("DETAIL LENS"))
    );

    application.handle_event(Event::Resize(Viewport::new(101, 30)));
    let first_wide = lines(&application.render());
    assert_eq!(
        [
            first_wide[2].trim_end(),
            first_wide[4].trim_end(),
            first_wide[5].trim_end(),
            first_wide[28].trim_end(),
            first_wide[29].trim_end(),
        ],
        [
            "  CHOOSE WHAT FITS RIGHT NOW                              │ DETAIL LENS / READ ONLY",
            "  > Mood & activity                                       │ DESTINATION",
            "    Radio stations                                        │ Mood & activity",
            " ↑/k up  ↓/j down  Enter open",
            " n queue  ? help  q quit",
        ]
    );
    assert!(first_wide[7].contains("Start from a Listening intent"));

    application.handle_event(Event::Key(Key::Down));
    let second_wide = lines(&application.render());
    assert!(second_wide[5].contains("Radio stations"));
    assert!(second_wide[7].contains("Browse continuous Stations"));
    assert_eq!(second_wide[28], first_wide[28]);
    assert_eq!(second_wide[29], first_wide[29]);
}

fn assert_wide_lens(application: &Application, expected: [&str; 3]) {
    let rendered = lines(&application.render());
    assert!(rendered[2].contains("DETAIL LENS / READ ONLY"));
    assert!(rendered[4].contains(expected[0]), "{}", rendered[4]);
    assert!(rendered[5].contains(expected[1]), "{}", rendered[5]);
    assert!(
        rendered[6..20]
            .iter()
            .any(|line| line.contains(expected[2])),
        "missing {:?} in lens: {rendered:#?}",
        expected[2]
    );
}

#[test]
fn every_top_level_browser_selection_has_useful_wide_details() {
    let mut intents = Application::new(Viewport::new(120, 40));
    intents.handle_event(Event::Key(Key::Enter));
    assert_wide_lens(&intents, ["LISTENING INTENT", "Deep Work", "8 Stations"]);

    let mut stations = Application::new(Viewport::new(120, 40));
    stations.handle_event(Event::Key(Key::Down));
    stations.handle_event(Event::Key(Key::Enter));
    assert_wide_lens(&stations, ["STATION", "Night Ledger", "Morrow Audio"]);

    let mut playlists = Application::new(Viewport::new(120, 40));
    for _ in 0..2 {
        playlists.handle_event(Event::Key(Key::Down));
    }
    playlists.handle_event(Event::Key(Key::Enter));
    assert_wide_lens(&playlists, ["PLAYLIST", "Deep Work Rotation", "20 Tracks"]);

    let mut services = Application::new(Viewport::new(120, 40));
    for _ in 0..3 {
        services.handle_event(Event::Key(Key::Down));
    }
    services.handle_event(Event::Key(Key::Enter));
    assert_wide_lens(&services, ["SERVICE", "Morrow Audio", "12 Tracks"]);
}

#[test]
fn nested_browsers_and_queue_keep_the_same_read_only_lens_contract() {
    let mut intent = Application::new(Viewport::new(120, 40));
    intent.handle_event(Event::Key(Key::Enter));
    intent.handle_event(Event::Key(Key::Enter));
    assert_wide_lens(&intent, ["STATION", "Night Ledger", "AVAILABLE"]);

    let mut service = Application::new(Viewport::new(120, 40));
    for _ in 0..3 {
        service.handle_event(Event::Key(Key::Down));
    }
    service.handle_event(Event::Key(Key::Enter));
    service.handle_event(Event::Key(Key::Enter));
    assert_wide_lens(&service, ["STATION", "Night Ledger", "Ambient"]);

    let mut search = Application::new(Viewport::new(120, 40));
    search.handle_event(Event::Key(Key::Char('/')));
    for character in "Night Geometry".chars() {
        search.handle_event(Event::Key(Key::Char(character)));
    }
    assert_wide_lens(&search, ["TRACK", "Night Geometry", "05:28"]);

    let mut playlist = Application::new(Viewport::new(120, 40));
    for _ in 0..2 {
        playlist.handle_event(Event::Key(Key::Down));
    }
    playlist.handle_event(Event::Key(Key::Enter));
    playlist.handle_event(Event::Key(Key::Enter));
    assert_wide_lens(&playlist, ["TRACK", "Night Geometry", "Sable Circuit"]);

    playlist.handle_event(Event::Key(Key::Escape));
    let request = playback_request(playlist.handle_event(Event::Key(Key::Char('p'))));
    playlist.handle_event(Event::PlaybackLoaded(request.id()));
    playlist.handle_event(Event::Key(Key::Char('n')));
    assert_wide_lens(&playlist, ["QUEUED TRACK", "Night Geometry", "HARBOR"]);
}

fn unicode_catalog() -> DemoCatalog {
    DemoCatalog::new(
        vec![Service::new("source", "מקור 音源", "源泉")],
        vec![],
        vec![Station::loading(
            "station",
            "深い集中のための非常に長い放送 e\u{301}lan שלום ".repeat(4),
            "静かな電子音楽",
            "結合文字 e\u{301}、漢字、右から左の文字 שלום を含む説明。",
            "source",
        )],
        vec![],
        vec![],
    )
}

fn open_only_service(application: &mut Application) {
    for _ in 0..3 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
}

#[test]
fn long_unicode_rows_preserve_state_status_and_source_rails_at_each_density() {
    for (viewport, source_open) in [
        (Viewport::new(80, 24), 74),
        (Viewport::new(100, 30), 94),
        (Viewport::new(120, 40), 52),
    ] {
        let mut application = Application::with_catalog(viewport, unicode_catalog());
        open_only_service(&mut application);

        let buffer = application.render();
        let row = &lines(&buffer)[6];

        assert!(row.starts_with("  SELECTED > STATION"), "{row:?}");
        assert!(
            row.contains('…'),
            "long title was not visibly truncated: {row:?}"
        );
        assert!(row.contains("LOADING"), "{row:?}");
        assert_eq!(buffer[(source_open, 6)].symbol(), "[");
        assert_eq!(buffer[(source_open + 1, 6)].symbol(), "源");
        assert_eq!(buffer[(source_open + 3, 6)].symbol(), "泉");
        assert_eq!(buffer[(source_open + 5, 6)].symbol(), "]");
        assert!(!row.contains('�'));
        if viewport == Viewport::new(120, 40) {
            let lens_title = &lines(&buffer)[5];
            assert!(lens_title.contains('…'), "{lens_title:?}");
            assert!(!lens_title.contains('�'));
        }
    }
}

#[test]
fn no_color_keeps_selection_and_state_text_without_explicit_color_cells() {
    let mut application =
        Application::with_options(Viewport::new(80, 24), ApplicationOptions::new(false, true));
    application.handle_event(Event::Key(Key::Down));
    let buffer = application.render();
    let rendered = lines(&buffer);

    assert!(rendered[5].contains("> Radio stations"));
    assert!(rendered[5].contains("SELECTED"));
    assert!(rendered[20].contains("Nothing playing"));
    assert!(buffer[(2, 5)].modifier.contains(Modifier::REVERSED));
    assert!(
        buffer
            .content
            .iter()
            .all(|cell| { matches!(cell.fg, Color::Reset) && matches!(cell.bg, Color::Reset) })
    );
}

#[test]
fn no_color_wide_selection_stops_before_the_detail_lens() {
    let home = Application::with_options(
        Viewport::new(120, 40),
        ApplicationOptions::new(false, false),
    );
    let mut intents = Application::with_options(
        Viewport::new(120, 40),
        ApplicationOptions::new(false, false),
    );
    intents.handle_event(Event::Key(Key::Enter));

    for (destination, application, selected_row) in
        [("Home", home, 4), ("Mood & activity", intents, 5)]
    {
        let buffer = application.render();

        for column in 2..58 {
            assert!(
                buffer[(column, selected_row)]
                    .modifier
                    .contains(Modifier::REVERSED),
                "{destination} selection ended before column {column}"
            );
        }
        for column in 58..120 {
            assert!(
                !buffer[(column, selected_row)]
                    .modifier
                    .contains(Modifier::REVERSED),
                "{destination} selection leaked into the detail lens at column {column}"
            );
        }
    }
}

#[test]
fn no_motion_freezes_loading_treatment_without_disabling_loading_feedback() {
    fn start_loading(options: ApplicationOptions) -> Application {
        let mut application = Application::with_options(Viewport::new(80, 24), options);
        application.handle_event(Event::Key(Key::Down));
        application.handle_event(Event::Key(Key::Enter));
        let request = playback_request(application.handle_event(Event::Key(Key::Char('p'))));
        assert!(lines(&application.render())[20].contains("Loading"));
        assert_eq!(
            request.target(),
            &songdial::PlaybackTarget::Station(songdial::CatalogId::new("morrow-night-ledger"))
        );
        application
    }

    let mut motion = start_loading(ApplicationOptions::new(true, true));
    let animated = motion.render();
    motion.handle_event(Event::Tick);
    assert_ne!(motion.render(), animated);

    let mut still = start_loading(ApplicationOptions::new(true, false));
    let no_motion = still.render();
    still.handle_event(Event::Tick);
    assert_eq!(still.render(), no_motion);
    assert!(lines(&no_motion)[20].contains("Loading"));
}

fn long_track_catalog() -> DemoCatalog {
    DemoCatalog::new(
        vec![Service::new("source", "מקור 音源", "源泉")],
        vec![],
        vec![],
        vec![Playlist::new(
            "playlist",
            "Long Queue",
            "Unicode rail fixture.",
            "source",
            &["track-one", "track-two"],
        )],
        vec![
            Track::new(
                "track-one",
                "深い集中 e\u{301}lan שלום ".repeat(5),
                "作曲家 Composer",
                "source",
                42,
            ),
            Track::new("track-two", "次の長い曲", "別の作曲家", "source", 125),
        ],
    )
}

#[test]
fn track_rows_protect_duration_and_source_before_truncating_unicode_titles() {
    for (viewport, duration_open, source_open) in [
        (Viewport::new(80, 24), 64, 74),
        (Viewport::new(120, 40), 42, 52),
    ] {
        let mut application = Application::with_catalog(viewport, long_track_catalog());
        for _ in 0..2 {
            application.handle_event(Event::Key(Key::Down));
        }
        application.handle_event(Event::Key(Key::Enter));
        application.handle_event(Event::Key(Key::Enter));

        let buffer = application.render();
        let row = &lines(&buffer)[7];
        assert!(row.starts_with("  SELECTED > TRACK"), "{row:?}");
        assert!(row.contains('…'), "{row:?}");
        assert_eq!(buffer[(duration_open, 7)].symbol(), "0");
        assert_eq!(buffer[(duration_open + 1, 7)].symbol(), "0");
        assert_eq!(buffer[(duration_open + 2, 7)].symbol(), ":");
        assert_eq!(buffer[(source_open, 7)].symbol(), "[");
        assert_eq!(buffer[(source_open + 1, 7)].symbol(), "源");
        assert_eq!(buffer[(source_open + 5, 7)].symbol(), "]");
    }
}

#[test]
fn tall_wide_browsers_use_the_available_rows_without_moving_the_persistent_frame() {
    let mut intent = Application::new(Viewport::new(120, 40));
    intent.handle_event(Event::Key(Key::Enter));
    intent.handle_event(Event::Key(Key::Enter));
    for _ in 0..12 {
        intent.handle_event(Event::Key(Key::Down));
    }
    let intent_lines = lines(&intent.render());
    assert!(intent_lines[6].contains("Night Ledger"));
    assert!(intent_lines[30].contains("SELECTED >"));
    assert!(intent_lines[30].contains("Empty Room"));
    assert!(intent_lines[36].starts_with(" NOW PLAYING"));

    let mut playlist = Application::new(Viewport::new(120, 40));
    for _ in 0..2 {
        playlist.handle_event(Event::Key(Key::Down));
    }
    playlist.handle_event(Event::Key(Key::Enter));
    playlist.handle_event(Event::Key(Key::Enter));
    for _ in 0..19 {
        playlist.handle_event(Event::Key(Key::Down));
    }
    let playlist_lines = lines(&playlist.render());
    assert!(playlist_lines[7].contains("Night Geometry"));
    assert!(playlist_lines[26].contains("SELECTED >"));
    assert!(playlist_lines[26].contains("Cinder Lines"));
    assert!(playlist_lines[36].starts_with(" NOW PLAYING"));
}

#[test]
fn resize_round_trips_preserve_exact_search_navigation_and_playback_session() {
    let mut application = Application::new(Viewport::new(80, 24));
    for _ in 0..2 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    let request = playback_request(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Key(Key::Char('/')));
    for character in "Night Geometry".chars() {
        application.handle_event(Event::Key(Key::Char(character)));
    }
    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Down));
    let compact = application.render();

    application.handle_event(Event::Resize(Viewport::new(120, 40)));
    let wide = lines(&application.render());
    assert!(wide[5].contains("Night Geometry"));
    assert!(wide[8].contains("Source  Harbor Sound [HARBOR]"));
    assert!(wide[36].contains("Night Geometry [MORROW] • PLAYING"));

    application.handle_event(Event::Resize(Viewport::new(80, 24)));
    assert_eq!(application.render(), compact);

    application.handle_event(Event::Resize(Viewport::new(79, 23)));
    application.handle_event(Event::Tick);
    application.handle_event(Event::Key(Key::Up));
    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Resize(Viewport::new(80, 24)));
    assert_eq!(application.render(), compact);
}

#[test]
fn playback_outcomes_received_during_the_guard_are_visible_after_recovery() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));
    let request = playback_request(application.handle_event(Event::Key(Key::Char('p'))));

    application.handle_event(Event::Resize(Viewport::new(79, 23)));
    assert_eq!(
        application.handle_event(Event::PlaybackLoaded(request.id())),
        Effect::None
    );
    application.handle_event(Event::Resize(Viewport::new(80, 24)));

    let recovered = lines(&application.render());
    assert!(recovered[20].contains("Night Ledger [MORROW] • LIVE"));
    assert!(!recovered[20].contains("Loading"));

    let mut failed = Application::new(Viewport::new(80, 24));
    failed.handle_event(Event::Key(Key::Down));
    failed.handle_event(Event::Key(Key::Enter));
    let request = playback_request(failed.handle_event(Event::Key(Key::Char('p'))));
    failed.handle_event(Event::Resize(Viewport::new(79, 23)));
    failed.handle_event(Event::PlaybackFailed {
        request_id: request.id(),
        reason: "The simulated signal could not load.".to_owned(),
    });
    failed.handle_event(Event::Resize(Viewport::new(80, 24)));

    let recovered = lines(&failed.render());
    assert!(recovered[21].contains("ERROR • Playback failed"));
    assert!(!recovered[20].contains("Loading"));
}

#[test]
fn loading_empty_unavailable_and_error_states_use_one_visible_vocabulary() {
    let mut loading = Application::new(Viewport::new(120, 40));
    loading.handle_event(Event::Key(Key::Down));
    loading.handle_event(Event::Key(Key::Enter));
    loading.handle_event(Event::Key(Key::Down));
    let loading_list = lines(&loading.render());
    assert!(loading_list[8].contains("LOADING"));
    assert!(loading_list[9].contains("Status  LOADING"));
    loading.handle_event(Event::Key(Key::Enter));
    assert_eq!(lines(&loading.render())[10].trim_end(), "  Status  LOADING");

    let mut unavailable = Application::new(Viewport::new(120, 40));
    unavailable.handle_event(Event::Key(Key::Down));
    unavailable.handle_event(Event::Key(Key::Enter));
    for _ in 0..6 {
        unavailable.handle_event(Event::Key(Key::Down));
    }
    let unavailable_list = lines(&unavailable.render());
    assert!(unavailable_list[18].contains("SEL+UNAV >"));
    assert!(unavailable_list[18].contains("UNAVAIL"));
    assert!(unavailable_list[9].contains("Status  UNAVAIL"));
    unavailable.handle_event(Event::Key(Key::Enter));
    let unavailable_detail = lines(&unavailable.render());
    assert_eq!(unavailable_detail[10].trim_end(), "  Status  UNAVAIL");
    assert!(unavailable_detail[11].contains("Signal maintenance"));

    let mut empty = Application::new(Viewport::new(120, 40));
    for _ in 0..2 {
        empty.handle_event(Event::Key(Key::Down));
    }
    empty.handle_event(Event::Key(Key::Enter));
    for _ in 0..4 {
        empty.handle_event(Event::Key(Key::Down));
    }
    let empty_list = lines(&empty.render());
    assert!(empty_list[14].contains("EMPTY"));
    assert!(empty_list[9].contains("Status  EMPTY"));
    empty.handle_event(Event::Key(Key::Enter));
    assert!(lines(&empty.render())[6].starts_with("  TRACKS • EMPTY"));

    let mut error = Application::new(Viewport::new(120, 40));
    error.handle_event(Event::Key(Key::Down));
    error.handle_event(Event::Key(Key::Enter));
    let request = playback_request(error.handle_event(Event::Key(Key::Char('p'))));
    error.handle_event(Event::PlaybackFailed {
        request_id: request.id(),
        reason: "The simulated signal could not load.".to_owned(),
    });
    error.handle_event(Event::Key(Key::Enter));
    assert_eq!(
        lines(&error.render())[37].trim_end(),
        "              ERROR • Playback failed: The simulated signal could not load."
    );
}

#[test]
fn stopped_playback_is_explicit_in_the_persistent_strip_and_wide_lens() {
    let catalog = DemoCatalog::new(
        vec![Service::new("source", "Local Source", "LOCAL")],
        vec![],
        vec![],
        vec![],
        vec![Track::new(
            "brief",
            "Brief Signal",
            "Test Tone",
            "source",
            1,
        )],
    );
    let mut application = Application::with_catalog(Viewport::new(120, 40), catalog);
    for _ in 0..3 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));
    let request = playback_request(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Tick);
    application.handle_event(Event::Key(Key::Char('n')));
    let stopped = lines(&application.render());

    assert!(stopped[3].contains("STOPPED"));
    assert!(stopped[36].contains("STOPPED"));
    assert!(stopped[11].contains("Playback  STOPPED"));
}

#[test]
fn stopped_playback_is_not_reported_as_playing_in_browser_rows() {
    let catalog = DemoCatalog::new(
        vec![Service::new("source", "Local Source", "LOCAL")],
        vec![],
        vec![],
        vec![Playlist::new(
            "playlist",
            "Brief Playlist",
            "One brief Track.",
            "source",
            &["brief"],
        )],
        vec![Track::new(
            "brief",
            "Brief Signal",
            "Test Tone",
            "source",
            1,
        )],
    );
    let mut application = Application::with_catalog(Viewport::new(120, 40), catalog);
    for _ in 0..2 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    let request = playback_request(application.handle_event(Event::Key(Key::Char('p'))));
    application.handle_event(Event::PlaybackLoaded(request.id()));
    application.handle_event(Event::Tick);

    let playlists = lines(&application.render());
    assert!(
        playlists[6].contains("SELECTED > PLAYLIST"),
        "{playlists:#?}"
    );
    assert!(!playlists[6].contains("SEL+PLAY"), "{playlists:#?}");
    assert!(!playlists[6].contains("PLAYING"), "{playlists:#?}");

    application.handle_event(Event::Key(Key::Enter));
    let tracks = lines(&application.render());
    assert!(tracks[7].contains("SELECTED > TRACK"), "{tracks:#?}");
    assert!(!tracks[7].contains("SEL+PLAY"), "{tracks:#?}");
    assert!(!tracks[7].contains("PLAYING"), "{tracks:#?}");
}

#[test]
fn playlist_track_lens_only_advertises_actions_that_work_there() {
    let mut application = Application::new(Viewport::new(120, 40));
    for _ in 0..2 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));
    application.handle_event(Event::Key(Key::Enter));

    let rendered = lines(&application.render());
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("p plays this Track")),
        "{rendered:#?}"
    );
    assert!(
        rendered
            .iter()
            .all(|line| !line.contains("Enter opens details")),
        "{rendered:#?}"
    );

    for _ in 0..15 {
        application.handle_event(Event::Key(Key::Down));
    }
    let unavailable = lines(&application.render());
    assert!(unavailable[10].contains("Status  UNAVAIL"));
    assert!(
        unavailable
            .iter()
            .any(|line| line.contains("p and a are unavailable for this Track.")),
        "{unavailable:#?}"
    );
    assert_eq!(unavailable[38].trim_end(), " ↑/k ↓/j move  p unavailable");
    assert_eq!(
        unavailable[39].trim_end(),
        " a unavailable  n queue  Esc back"
    );
}

#[test]
fn search_service_and_catalog_rows_share_the_protected_rails() {
    let mut search = Application::new(Viewport::new(120, 40));
    search.handle_event(Event::Key(Key::Char('/')));
    for character in "Night Geometry".chars() {
        search.handle_event(Event::Key(Key::Char(character)));
    }
    let search_buffer = search.render();
    assert_eq!(search_buffer[(42, 7)].symbol(), "0");
    assert_eq!(search_buffer[(44, 7)].symbol(), ":");
    assert_eq!(search_buffer[(50, 7)].symbol(), "[");
    assert_eq!(search_buffer[(57, 7)].symbol(), "]");

    let mut service = Application::new(Viewport::new(120, 40));
    for _ in 0..3 {
        service.handle_event(Event::Key(Key::Down));
    }
    service.handle_event(Event::Key(Key::Enter));
    let service_buffer = service.render();
    assert_eq!(service_buffer[(40, 6)].symbol(), "C");
    assert_eq!(service_buffer[(50, 6)].symbol(), "[");
    assert_eq!(service_buffer[(57, 6)].symbol(), "]");

    service.handle_event(Event::Key(Key::Enter));
    for _ in 0..6 {
        service.handle_event(Event::Key(Key::Down));
    }
    let catalog_buffer = service.render();
    assert_eq!(catalog_buffer[(42, 18)].symbol(), "0");
    assert_eq!(catalog_buffer[(44, 18)].symbol(), ":");
    assert_eq!(catalog_buffer[(50, 18)].symbol(), "[");
    assert_eq!(catalog_buffer[(57, 18)].symbol(), "]");
}

#[test]
fn long_now_playing_titles_yield_before_source_state_and_progress() {
    for viewport in [Viewport::new(80, 24), Viewport::new(120, 40)] {
        let mut application = Application::with_catalog(viewport, long_track_catalog());
        for _ in 0..2 {
            application.handle_event(Event::Key(Key::Down));
        }
        application.handle_event(Event::Key(Key::Enter));
        let request = playback_request(application.handle_event(Event::Key(Key::Char('p'))));

        let loading = lines(&application.render());
        let now_playing_row = usize::from(viewport == Viewport::new(120, 40)) * 16 + 20;
        assert!(loading[now_playing_row].contains("Loading"));
        assert!(loading[now_playing_row].contains('源'));
        assert!(loading[now_playing_row].contains('泉'));
        assert!(loading[now_playing_row].contains('…'));

        application.handle_event(Event::PlaybackLoaded(request.id()));
        let playing = lines(&application.render());
        assert!(playing[now_playing_row].contains('…'));
        assert!(
            playing[now_playing_row].contains("PLAYING • 00:00/00:42")
                && playing[now_playing_row].contains('源')
                && playing[now_playing_row].contains('泉'),
            "{:?}",
            playing[now_playing_row]
        );
    }
}

#[test]
fn long_unicode_breadcrumbs_preserve_both_root_and_current_location() {
    let mut application = Application::with_catalog(Viewport::new(80, 24), unicode_catalog());
    open_only_service(&mut application);
    application.handle_event(Event::Key(Key::Enter));
    let location = lines(&application.render())[0].trim_end().to_owned();

    assert!(location.starts_with(" SONGDIAL / "), "{location:?}");
    assert!(location.contains('…'), "{location:?}");
    assert!(location.ends_with("שלום"), "{location:?}");
    assert!(!location.contains('�'));
}

fn empty_catalog() -> DemoCatalog {
    DemoCatalog::new(vec![], vec![], vec![], vec![], vec![])
}

#[test]
fn every_empty_browser_explains_the_state_and_a_recovery_action() {
    let expectations = [
        (
            0,
            "0 Listening intents • EMPTY",
            "No Listening intents",
            "Return Home",
        ),
        (1, "0 Stations • EMPTY", "No Stations", "Return Home"),
        (2, "0 Playlists • EMPTY", "No Playlists", "Return Home"),
        (3, "0 Services • EMPTY", "No Services", "Return Home"),
        (4, "Start typing", "Start typing", "Type a query"),
    ];

    for (home_index, body, lens_title, recovery) in expectations {
        let mut application = Application::with_catalog(Viewport::new(120, 40), empty_catalog());
        for _ in 0..home_index {
            application.handle_event(Event::Key(Key::Down));
        }
        application.handle_event(Event::Key(Key::Enter));
        let rendered = lines(&application.render());

        assert!(
            rendered.iter().any(|line| line.contains(body)),
            "{rendered:#?}"
        );
        assert!(rendered[5].contains(lens_title), "{rendered:#?}");
        assert!(
            rendered[6..16].iter().any(|line| line.contains(recovery)),
            "{rendered:#?}"
        );
    }
}

#[test]
fn nested_empty_destinations_keep_the_same_wide_lens_contract() {
    let mut reset = Application::new(Viewport::new(120, 40));
    reset.handle_event(Event::Key(Key::Enter));
    for _ in 0..5 {
        reset.handle_event(Event::Key(Key::Down));
    }
    reset.handle_event(Event::Key(Key::Enter));
    let reset_lines = lines(&reset.render());
    assert!(reset_lines[4].contains("0 Stations • 0 Playlists • EMPTY"));
    assert!(reset_lines[5].contains("Reset"));
    assert!(reset_lines[7].contains("No matches"));

    let mut playlist = Application::new(Viewport::new(120, 40));
    for _ in 0..2 {
        playlist.handle_event(Event::Key(Key::Down));
    }
    playlist.handle_event(Event::Key(Key::Enter));
    for _ in 0..4 {
        playlist.handle_event(Event::Key(Key::Down));
    }
    playlist.handle_event(Event::Key(Key::Enter));
    let playlist_lines = lines(&playlist.render());
    assert!(playlist_lines[6].contains("TRACKS • EMPTY"));
    assert!(playlist_lines[5].contains("Empty Room"));
    assert!(playlist_lines[9].contains("Status  EMPTY"));

    let catalog = DemoCatalog::new(
        vec![Service::new("empty", "Empty Service", "EMPTY")],
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut service = Application::with_catalog(Viewport::new(120, 40), catalog);
    open_only_service(&mut service);
    let service_lines = lines(&service.render());
    assert!(service_lines[4].contains("0 Stations • 0 Playlists • 0 Tracks • EMPTY"));
    assert!(service_lines[5].contains("Empty Service"));
    assert!(service_lines[7].contains("No catalog items"));
}

#[test]
fn wide_search_uses_available_rows_and_keeps_group_and_scroll_indicators() {
    let mut application = Application::new(Viewport::new(120, 40));
    application.handle_event(Event::Key(Key::Char('/')));
    for character in "MORROW".chars() {
        application.handle_event(Event::Key(Key::Char(character)));
    }
    for _ in 0..30 {
        application.handle_event(Event::Key(Key::Down));
    }
    let rendered = lines(&application.render());

    assert!(rendered[4].starts_with("  18 results • Result 18/18"));
    assert!(rendered[6].contains("STATIONS • 4"));
    assert!(rendered[7].contains("Night Ledger"));
    assert!(rendered[14].contains("TRACKS • 12"));
    assert!(rendered[26].contains("SELECTED >"));
    assert!(rendered[26].contains("Long Form"));
    assert!(rendered[36].starts_with(" NOW PLAYING"));
}
