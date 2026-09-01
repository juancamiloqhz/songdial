use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use songdial::{Application, Effect, Event, Key, Viewport};

const TICK_RATE: Duration = Duration::from_secs(1);

pub fn run(_no_motion: bool) -> io::Result<()> {
    let _session = TerminalSession::enter()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let size = terminal.size()?;
    let mut application = Application::new(Viewport::new(size.width, size.height));
    let mut next_tick = Instant::now() + TICK_RATE;

    loop {
        terminal.draw(|frame| frame.buffer_mut().merge(&application.render()))?;

        let wait = next_tick.saturating_duration_since(Instant::now());
        if event::poll(wait)? {
            let application_event = match event::read()? {
                CrosstermEvent::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
                    translate_key(key).map(Event::Key)
                }
                CrosstermEvent::Resize(width, height) => {
                    Some(Event::Resize(Viewport::new(width, height)))
                }
                _ => None,
            };

            if application_event
                .is_some_and(|event| application.handle_event(event) == Effect::Quit)
            {
                break;
            }
        }

        if Instant::now() >= next_tick {
            if application.handle_event(Event::Tick) == Effect::Quit {
                break;
            }
            next_tick = Instant::now() + TICK_RATE;
        }
    }

    Ok(())
}

fn translate_key(key: KeyEvent) -> Option<Key> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Key::CtrlC)
        }
        (KeyCode::Up, _) => Some(Key::Up),
        (KeyCode::Down, _) => Some(Key::Down),
        (KeyCode::Enter, _) => Some(Key::Enter),
        (KeyCode::Esc, _) => Some(Key::Escape),
        (KeyCode::Char(character), modifiers)
            if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            Some(Key::Char(character))
        }
        _ => None,
    }
}

#[derive(Default)]
struct TerminalSession {
    raw_mode: bool,
    alternate_screen: bool,
    cursor_hidden: bool,
}

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        let mut session = Self::default();

        enable_raw_mode()?;
        session.raw_mode = true;

        session.alternate_screen = true;
        execute!(stdout(), EnterAlternateScreen)?;

        session.cursor_hidden = true;
        execute!(stdout(), Hide)?;

        Ok(session)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if self.cursor_hidden {
            let _ = execute!(stdout(), Show);
        }
        if self.alternate_screen {
            let _ = execute!(stdout(), LeaveAlternateScreen);
        }
        if self.raw_mode {
            let _ = disable_raw_mode();
        }
    }
}
