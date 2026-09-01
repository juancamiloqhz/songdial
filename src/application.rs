use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HomeChoice {
    ListeningIntents,
    Stations,
    Playlists,
    Services,
    Search,
}

impl HomeChoice {
    const ALL: [Self; 5] = [
        Self::ListeningIntents,
        Self::Stations,
        Self::Playlists,
        Self::Services,
        Self::Search,
    ];

    const fn text(self) -> HomeChoiceText {
        match self {
            Self::ListeningIntents => HomeChoiceText {
                label: "Mood & activity",
                title: "MOOD & ACTIVITY",
            },
            Self::Stations => HomeChoiceText {
                label: "Radio stations",
                title: "RADIO STATIONS",
            },
            Self::Playlists => HomeChoiceText {
                label: "My playlists",
                title: "MY PLAYLISTS",
            },
            Self::Services => HomeChoiceText {
                label: "Browse services",
                title: "BROWSE SERVICES",
            },
            Self::Search => HomeChoiceText {
                label: "Search everything",
                title: "SEARCH EVERYTHING",
            },
        }
    }
}

struct HomeChoiceText {
    label: &'static str,
    title: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    width: u16,
    height: u16,
}

impl Viewport {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Up,
    Down,
    Enter,
    Escape,
    CtrlC,
    Char(char),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    Key(Key),
    Tick,
    Resize(Viewport),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    None,
    Quit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Destination {
    Home,
    NotYetAvailable(HomeChoice),
}

pub struct Application {
    viewport: Viewport,
    selection: usize,
    destination: Destination,
    help_visible: bool,
}

impl Application {
    #[must_use]
    pub const fn new(viewport: Viewport) -> Self {
        Self {
            viewport,
            selection: 0,
            destination: Destination::Home,
            help_visible: false,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Effect {
        if let Event::Resize(viewport) = event {
            self.viewport = viewport;
            return Effect::None;
        }

        if event == Event::Tick {
            return Effect::None;
        }

        if matches!(event, Event::Key(Key::Char('q') | Key::CtrlC)) {
            return Effect::Quit;
        }

        if !self.viewport.is_supported() {
            return Effect::None;
        }

        if event == Event::Key(Key::Char('?')) {
            self.help_visible = !self.help_visible;
            return Effect::None;
        }

        if self.help_visible && event == Event::Key(Key::Escape) {
            self.help_visible = false;
            return Effect::None;
        }

        match (self.destination, event) {
            (Destination::Home, Event::Key(Key::Down | Key::Char('j'))) => {
                self.selection = (self.selection + 1).min(HomeChoice::ALL.len() - 1);
            }
            (Destination::Home, Event::Key(Key::Up | Key::Char('k'))) => {
                self.selection = self.selection.saturating_sub(1);
            }
            (Destination::Home, Event::Key(Key::Enter)) => {
                self.destination = Destination::NotYetAvailable(self.selected_home_choice());
            }
            (Destination::NotYetAvailable(_), Event::Key(Key::Escape)) => {
                self.destination = Destination::Home;
            }
            (_, Event::Key(_)) => {}
            (_, Event::Tick) => unreachable!("tick events return before navigation"),
            (_, Event::Resize(_)) => unreachable!("resize events return before navigation"),
        }

        Effect::None
    }

    #[must_use]
    pub fn render(&self) -> Buffer {
        let area = Rect::new(0, 0, self.viewport.width, self.viewport.height);
        let mut buffer = Buffer::empty(area);
        let base = Style::default()
            .fg(Color::Rgb(222, 216, 202))
            .bg(Color::Rgb(27, 29, 28));
        buffer.set_style(area, base);

        if !self.viewport.is_supported() {
            return buffer;
        }

        if self.help_visible {
            self.render_help(&mut buffer, base);
        } else {
            self.render_destination(&mut buffer, base);
        }

        buffer
    }

    fn render_destination(&self, buffer: &mut Buffer, base: Style) {
        match self.destination {
            Destination::Home => self.render_home(buffer, base),
            Destination::NotYetAvailable(choice) => {
                let title = choice.text().title;
                buffer.set_string(0, 0, format!(" SONGDIAL / {title}"), base);
                buffer.set_string(0, 2, format!("  {title}"), base);
                buffer.set_string(0, 4, "  This Destination is not yet available.", base);
                buffer.set_string(
                    0,
                    5,
                    "  Return Home to choose another listening path.",
                    base,
                );
                self.render_now_playing(buffer, base);
                buffer.set_string(0, self.guide_top(), " Esc back  ? help  q quit", base);
            }
        }
    }

    fn render_home(&self, buffer: &mut Buffer, base: Style) {
        buffer.set_string(0, 0, " SONGDIAL / HOME", base);
        buffer.set_string(0, 2, "  CHOOSE WHAT FITS RIGHT NOW", base);
        for (index, choice) in HomeChoice::ALL.iter().enumerate() {
            let label = choice.text().label;
            let line = if index == self.selection {
                format!("  > {label:<58}SELECTED")
            } else {
                format!("    {label}")
            };
            buffer.set_string(0, 4 + index as u16, line, base);
        }
        buffer.set_string(
            0,
            10,
            "  Enter opens a Destination. Playback always starts separately.",
            base,
        );
        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " ↑/k up  ↓/j down  Enter open", base);
        buffer.set_string(0, self.guide_top() + 1, " ? help  q quit", base);

        let selected = Style::default()
            .fg(Color::Rgb(27, 29, 28))
            .bg(Color::Rgb(214, 166, 75))
            .add_modifier(Modifier::BOLD);
        buffer.set_style(Rect::new(2, 4 + self.selection as u16, 76, 1), selected);
    }

    fn render_now_playing(&self, buffer: &mut Buffer, base: Style) {
        let top = self.now_playing_top();
        buffer.set_string(0, top, " NOW PLAYING  Nothing playing", base);
        buffer.set_string(
            0,
            top + 1,
            "              Open a choice to keep exploring.",
            base,
        );
    }

    fn render_help(&self, buffer: &mut Buffer, base: Style) {
        let destination = match self.destination {
            Destination::Home => "HOME",
            Destination::NotYetAvailable(choice) => choice.text().title,
        };
        buffer.set_string(0, 0, format!(" SONGDIAL / {destination} / HELP"), base);
        buffer.set_string(0, 2, "  COMPLETE KEY GUIDE", base);
        buffer.set_string(0, 4, "  ↑ / k    Move selection up", base);
        buffer.set_string(0, 5, "  ↓ / j    Move selection down", base);
        buffer.set_string(0, 6, "  Enter    Open without playing", base);
        buffer.set_string(0, 7, "  Esc      Go back or close help", base);
        buffer.set_string(0, 8, "  ?        Show contextual help", base);
        buffer.set_string(0, 9, "  q        Quit", base);
        buffer.set_string(0, 10, "  Ctrl+C   Quit immediately", base);
        let local_help = match self.destination {
            Destination::Home => "  Home: choose a listening path, then press Enter.",
            Destination::NotYetAvailable(_) => "  This Destination has no additional actions yet.",
        };
        buffer.set_string(0, 12, local_help, base);
        self.render_now_playing(buffer, base);
        buffer.set_string(0, self.guide_top(), " Esc close  ? close  q quit", base);
    }

    const fn now_playing_top(&self) -> u16 {
        self.viewport.height - 4
    }

    const fn guide_top(&self) -> u16 {
        self.viewport.height - 2
    }

    fn selected_home_choice(&self) -> HomeChoice {
        HomeChoice::ALL[self.selection]
    }
}

impl Viewport {
    const fn is_supported(self) -> bool {
        self.width >= 80 && self.height >= 24
    }
}
