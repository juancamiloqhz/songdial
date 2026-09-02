use songdial::{Application, Event, Key, Viewport};

mod support;

use support::lines;

fn compact_line(content: &str) -> String {
    format!("{content:<80}")
}

#[test]
fn launches_home_in_the_exact_compact_frame() {
    let application = Application::new(Viewport::new(80, 24));

    let rendered = application.render();

    let expected = [
        " SONGDIAL / HOME",
        "",
        "  CHOOSE WHAT FITS RIGHT NOW",
        "",
        "  > Mood & activity                                           SELECTED",
        "    Radio stations",
        "    My playlists",
        "    Browse services",
        "    Search everything",
        "",
        "  Enter opens a Destination. Playback always starts separately.",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        " NOW PLAYING  Nothing playing",
        "              Open a choice to keep exploring.",
        " ↑/k up  ↓/j down  Enter open",
        " n queue  ? help  q quit",
    ]
    .map(compact_line);

    assert_eq!(lines(&rendered), expected);
}

#[test]
fn arrow_and_vim_keys_move_home_selection_in_display_order() {
    let mut application = Application::new(Viewport::new(80, 24));

    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Char('j')));
    application.handle_event(Event::Key(Key::Char('k')));

    let rendered = lines(&application.render());
    let home_rows = rendered[4..9]
        .iter()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>();

    assert_eq!(
        home_rows,
        [
            "    Mood & activity",
            "  > Radio stations                                            SELECTED",
            "    My playlists",
            "    Browse services",
            "    Search everything",
        ]
    );
}

#[test]
fn enter_opens_every_home_choice_without_starting_playback() {
    let choices = [
        "MOOD & ACTIVITY",
        "RADIO STATIONS",
        "MY PLAYLISTS",
        "BROWSE SERVICES",
        "SEARCH EVERYTHING",
    ];
    let mut opened = Vec::new();

    for (index, choice) in choices.iter().enumerate() {
        let mut application = Application::new(Viewport::new(80, 24));
        for _ in 0..index {
            application.handle_event(Event::Key(Key::Down));
        }

        application.handle_event(Event::Key(Key::Enter));
        let rendered = lines(&application.render());
        let body_row = if index == 0 { 2 } else { 4 };
        opened.push((
            rendered[0].trim_end().to_owned(),
            rendered[body_row].trim_end().to_owned(),
            rendered[20].trim_end().to_owned(),
            rendered[22].trim_end().to_owned(),
            *choice,
        ));
    }

    let expected = choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let (body, guide) = if index == 0 {
                (
                    "  MOOD & ACTIVITY".to_owned(),
                    " ↑/k up  ↓/j down  Enter open".to_owned(),
                )
            } else {
                (
                    "  This Destination is not yet available.".to_owned(),
                    " n queue  Esc back  ? help  q quit".to_owned(),
                )
            };
            (
                format!(" SONGDIAL / {choice}"),
                body,
                " NOW PLAYING  Nothing playing".to_owned(),
                guide,
                *choice,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(opened, expected);
}

#[test]
fn escape_restores_the_selected_home_choice_and_is_harmless_on_home() {
    let mut application = Application::new(Viewport::new(80, 24));
    for _ in 0..3 {
        application.handle_event(Event::Key(Key::Down));
    }
    application.handle_event(Event::Key(Key::Enter));

    application.handle_event(Event::Key(Key::Escape));
    application.handle_event(Event::Key(Key::Escape));

    let rendered = lines(&application.render());
    assert_eq!(
        (
            rendered[0].trim_end(),
            rendered[7].trim_end(),
            rendered[22].trim_end(),
        ),
        (
            " SONGDIAL / HOME",
            "  > Browse services                                           SELECTED",
            " ↑/k up  ↓/j down  Enter open",
        )
    );
}

#[test]
fn help_lists_global_and_destination_keys_without_losing_context() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Down));
    application.handle_event(Event::Key(Key::Enter));

    application.handle_event(Event::Key(Key::Char('?')));
    let help = lines(&application.render());
    let help_excerpt = [0, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 20, 22]
        .map(|row| help[row].trim_end().to_owned());

    application.handle_event(Event::Key(Key::Escape));
    let restored = lines(&application.render());

    assert_eq!(
        (help_excerpt, restored[0].trim_end()),
        (
            [
                " SONGDIAL / RADIO STATIONS / HELP".to_owned(),
                "  COMPLETE KEY GUIDE".to_owned(),
                "  ↑ / k    Move selection up".to_owned(),
                "  ↓ / j    Move selection down".to_owned(),
                "  Enter    Open without playing".to_owned(),
                "  p        Start a new Playback session".to_owned(),
                "  Space    Pause, resume, or restart".to_owned(),
                "  a        Add a Track or Playlist to the Queue".to_owned(),
                "  d        Remove the selected queued Track".to_owned(),
                "  n        Open Now Playing and Queue".to_owned(),
                "  Esc      Go back or close help".to_owned(),
                "  ?        Show contextual help".to_owned(),
                "  q / Ctrl+C  Quit".to_owned(),
                "  This Destination has no additional actions yet.".to_owned(),
                " NOW PLAYING  Nothing playing".to_owned(),
                " Esc close  ? close  q quit".to_owned(),
            ],
            " SONGDIAL / RADIO STATIONS"
        )
    );
}

#[test]
fn quit_keys_expose_a_runtime_effect_from_every_context() {
    let mut home = Application::new(Viewport::new(80, 24));

    let mut destination = Application::new(Viewport::new(80, 24));
    destination.handle_event(Event::Key(Key::Enter));

    let mut help = Application::new(Viewport::new(80, 24));
    help.handle_event(Event::Key(Key::Char('?')));

    assert_eq!(
        [
            home.handle_event(Event::Key(Key::Char('q'))),
            destination.handle_event(Event::Key(Key::Char('q'))),
            help.handle_event(Event::Key(Key::CtrlC)),
        ],
        [
            songdial::Effect::Quit,
            songdial::Effect::Quit,
            songdial::Effect::Quit,
        ]
    );
}

#[test]
fn deterministic_tick_events_leave_idle_home_unchanged() {
    let mut application = Application::new(Viewport::new(80, 24));
    let before = application.render();

    let effect = application.handle_event(Event::Tick);

    assert_eq!(
        (effect, application.render()),
        (songdial::Effect::None, before)
    );
}

#[test]
fn resize_events_update_the_viewport_without_resetting_selection() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Resize(Viewport::new(81, 24)));
    let rendered = application.render();
    let rendered_lines = lines(&rendered);

    assert_eq!(
        (
            rendered.area.width,
            rendered.area.height,
            rendered_lines[0].trim_end(),
            rendered_lines[5].trim_end(),
            rendered_lines[20].trim_end(),
            rendered_lines[21].trim_end(),
            rendered_lines[22].trim_end(),
            rendered_lines[23].trim_end(),
        ),
        (
            81,
            24,
            " SONGDIAL / HOME",
            "  > Radio stations                                            SELECTED",
            " NOW PLAYING  Nothing playing",
            "              Open a choice to keep exploring.",
            " ↑/k up  ↓/j down  Enter open",
            " n queue  ? help  q quit",
        )
    );
}

#[test]
fn unsupported_viewport_is_safe_without_implementing_the_deferred_guard() {
    let mut application = Application::new(Viewport::new(80, 24));
    application.handle_event(Event::Key(Key::Down));

    application.handle_event(Event::Resize(Viewport::new(1, 1)));
    let rendered = application.render();
    let quit = application.handle_event(Event::Key(Key::Char('q')));

    assert_eq!(
        (
            rendered.area.width,
            rendered.area.height,
            lines(&rendered),
            quit,
        ),
        (1, 1, vec![" ".to_owned()], songdial::Effect::Quit)
    );
}
