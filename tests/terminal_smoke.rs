use std::{
    ffi::OsStr,
    io::{Read, Write},
    sync::mpsc::{self, Receiver},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system};

const SCREEN_TIMEOUT: Duration = Duration::from_secs(5);

struct PtySession {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    writer: Option<Box<dyn Write + Send>>,
    output_receiver: Receiver<Vec<u8>>,
    reader_thread: Option<JoinHandle<()>>,
    terminal_before: String,
    output: Vec<u8>,
}

struct CompletedPty {
    status: ExitStatus,
    terminal_before: String,
    terminal_after: String,
    output: String,
}

impl CompletedPty {
    fn assert_terminal_restored(&self) {
        assert_eq!(self.terminal_after, self.terminal_before);
    }

    fn assert_screen_lifecycle(&self) {
        for sequence in [
            "\u{1b}[?1049h",
            "\u{1b}[?25l",
            "\u{1b}[?25h",
            "\u{1b}[?1049l",
        ] {
            assert!(
                self.output.contains(sequence),
                "PTY output was: {}",
                self.output.escape_debug()
            );
        }
    }
}

impl PtySession {
    fn spawn(command: CommandBuilder) -> Self {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("test PTY should open");
        let terminal_before = format!("{:?}", pair.master.get_termios());
        let child = pair
            .slave
            .spawn_command(command)
            .expect("songdial should start in a PTY");
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .expect("PTY output should be readable");
        let writer = pair
            .master
            .take_writer()
            .expect("PTY input should be writable");
        let (output_sender, output_receiver) = mpsc::channel();
        let reader_thread = std::thread::spawn(move || {
            let mut chunk = [0_u8; 4096];
            while let Ok(read) = reader.read(&mut chunk) {
                if read == 0 || output_sender.send(chunk[..read].to_vec()).is_err() {
                    break;
                }
            }
        });

        Self {
            master: pair.master,
            child,
            writer: Some(writer),
            output_receiver,
            reader_thread: Some(reader_thread),
            terminal_before,
            output: Vec::new(),
        }
    }

    fn send(&mut self, input: &[u8]) {
        let writer = self.writer.as_mut().expect("PTY writer should be open");
        writer.write_all(input).expect("PTY input should be sent");
        writer.flush().expect("PTY input should be flushed");
    }

    fn resize(&self, columns: u16, rows: u16) {
        self.master
            .resize(PtySize {
                rows,
                cols: columns,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("PTY should resize");
    }

    fn output_stage(&mut self) -> usize {
        self.drain_available_output();
        self.output.len()
    }

    fn wait_for_screen_state(&mut self, stage_start: usize, expected: &str) {
        if self.output_since(stage_start).contains(expected) {
            return;
        }

        let deadline = Instant::now() + SCREEN_TIMEOUT;
        while Instant::now() < deadline {
            if let Ok(chunk) = self.output_receiver.recv_timeout(Duration::from_millis(25)) {
                self.output.extend(chunk);
                if self.output_since(stage_start).contains(expected) {
                    return;
                }
            }
            if let Some(status) = self
                .child
                .try_wait()
                .expect("child status should be readable")
            {
                panic!(
                    "songdial exited {status} before {expected:?}; PTY output was: {}",
                    self.output_since(stage_start).escape_debug()
                );
            }
        }

        panic!(
            "did not observe {expected:?}; PTY output was: {}",
            self.output_since(stage_start).escape_debug()
        );
    }

    fn is_running(&mut self) -> bool {
        self.child
            .try_wait()
            .expect("child status should be readable")
            .is_none()
    }

    fn wait_for_exit(&mut self, fallback_input: Option<&[u8]>) -> ExitStatus {
        let deadline = Instant::now() + SCREEN_TIMEOUT;
        loop {
            self.drain_available_output();
            if let Some(status) = self
                .child
                .try_wait()
                .expect("child status should be readable")
            {
                return status;
            }
            if Instant::now() >= deadline {
                if let Some(input) = fallback_input {
                    self.send(input);
                } else {
                    self.child.kill().expect("hung child should be stopped");
                }
                return self.child.wait().expect("child should be reaped");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn finish(mut self, status: ExitStatus) -> CompletedPty {
        drop(self.writer.take());
        while let Ok(chunk) = self.output_receiver.recv_timeout(Duration::from_millis(25)) {
            self.output.extend(chunk);
        }
        self.reader_thread
            .take()
            .expect("PTY reader thread should exist")
            .join()
            .expect("PTY reader should finish");

        CompletedPty {
            status,
            terminal_before: self.terminal_before,
            terminal_after: format!("{:?}", self.master.get_termios()),
            output: String::from_utf8_lossy(&self.output).into_owned(),
        }
    }

    fn drain_available_output(&mut self) {
        while let Ok(chunk) = self.output_receiver.try_recv() {
            self.output.extend(chunk);
        }
    }

    fn output_since(&self, stage_start: usize) -> String {
        String::from_utf8_lossy(&self.output[stage_start..]).into_owned()
    }
}

fn command_for(binary: impl AsRef<OsStr>) -> CommandBuilder {
    let mut command = CommandBuilder::new(binary);
    command.arg("--no-motion");
    command.env("TERM", "xterm-256color");
    command
}

fn smoke_binary() -> std::ffi::OsString {
    std::env::var_os("SONGDIAL_SMOKE_BINARY")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_songdial").into())
}

fn assert_home_and_restoration_after(quit_key: &[u8], no_color: bool) {
    let mut command = command_for(smoke_binary());
    if no_color {
        command.env("NO_COLOR", "1");
    }
    let mut session = PtySession::spawn(command);
    session.wait_for_screen_state(0, "Mood & activity");
    session.send(quit_key);

    let status = session.wait_for_exit(None);
    let completed = session.finish(status);
    assert!(
        completed.status.success(),
        "PTY output was: {}",
        completed.output.escape_debug()
    );
    completed.assert_terminal_restored();
    completed.assert_screen_lifecycle();

    if no_color {
        assert!(
            !completed.output.contains("38;2;") && !completed.output.contains("48;2;"),
            "PTY output was: {}",
            completed.output.escape_debug()
        );
    }
}

#[test]
fn real_binary_shows_home_quits_and_restores_the_terminal() {
    assert_home_and_restoration_after(b"q", false);
}

#[test]
fn control_c_exits_immediately_and_restores_the_terminal() {
    assert_home_and_restoration_after(b"\x03", false);
}

#[test]
fn no_color_reaches_the_application_without_emitting_rgb_sequences() {
    assert_home_and_restoration_after(b"q", true);
}

#[test]
fn startup_failure_after_raw_mode_restores_the_terminal() {
    let mut command = command_for(env!("CARGO_BIN_EXE_songdial"));
    command.env("SONGDIAL_TEST_FAIL_STARTUP_AFTER_RAW_MODE", "1");
    let mut session = PtySession::spawn(command);

    let status = session.wait_for_exit(None);
    let completed = session.finish(status);
    completed.assert_terminal_restored();
    assert!(
        !completed.status.success()
            && completed
                .output
                .contains("error: injected startup failure after raw mode"),
        "PTY output was: {}",
        completed.output.escape_debug()
    );
}

#[test]
fn handled_runtime_failure_restores_the_terminal() {
    let mut command = command_for(env!("CARGO_BIN_EXE_songdial"));
    command.env("SONGDIAL_TEST_FAIL_RUNTIME_AFTER_DRAW", "1");
    let mut session = PtySession::spawn(command);

    let status = session.wait_for_exit(Some(b"q"));
    let completed = session.finish(status);
    completed.assert_terminal_restored();
    completed.assert_screen_lifecycle();
    assert!(
        !completed.status.success()
            && completed.output.contains("Mood & activity")
            && completed
                .output
                .contains("error: injected runtime failure after draw"),
        "PTY output was: {}",
        completed.output.escape_debug()
    );
}

#[test]
fn packaged_binary_completes_a_meaningful_smoke_journey_and_restores_the_terminal() {
    let mut command = CommandBuilder::new(smoke_binary());
    command.arg("--no-motion");
    command.env_clear();
    command.env("TERM", "xterm-256color");
    let mut session = PtySession::spawn(command);

    session.wait_for_screen_state(0, "Mood & activity");

    let stage = session.output_stage();
    session.send(b"\r");
    session.wait_for_screen_state(stage, "Deep Work");

    let stage = session.output_stage();
    session.send(b"/");
    session.wait_for_screen_state(stage, "Query >");

    let stage = session.output_stage();
    session.send(b"Night");
    session.wait_for_screen_state(stage, "Night Geometry");

    let stage = session.output_stage();
    session.send(b"\x1b");
    session.wait_for_screen_state(stage, "edit query");

    let stage = session.output_stage();
    session.send(b"\x1b");
    session.wait_for_screen_state(stage, "Deep Work");

    let stage = session.output_stage();
    session.send(b"\x1b");
    session.wait_for_screen_state(stage, "Mood & activity");

    let stage = session.output_stage();
    session.send(b"jj\r");
    session.wait_for_screen_state(stage, "MY PLAYLISTS");

    let stage = session.output_stage();
    session.send(b"p");
    session.wait_for_screen_state(stage, "Night Geometry [MORROW] • PLAYING");

    let stage = session.output_stage();
    session.send(b"n");
    session.wait_for_screen_state(stage, "CURRENT ");

    let stage = session.output_stage();
    session.resize(81, 24);
    assert_queue_state(&mut session, stage);

    let stage = session.output_stage();
    session.resize(79, 23);
    session.wait_for_screen_state(stage, "Required 80×24 cells");

    let stage = session.output_stage();
    session.resize(80, 24);
    assert_queue_state(&mut session, stage);

    let stage = session.output_stage();
    session.send(b"q");
    session.wait_for_screen_state(stage, "QUIT");
    assert!(
        session.is_running(),
        "active playback should require quit confirmation"
    );

    let stage = session.output_stage();
    session.send(b"\x1b");
    // Confirm Esc was consumed before another key can be parsed with it as an Alt chord.
    session.wait_for_screen_state(stage, "Stillwater");

    let stage = session.output_stage();
    session.resize(81, 24);
    assert_queue_state(&mut session, stage);

    let stage = session.output_stage();
    session.send(b"q");
    session.wait_for_screen_state(stage, "QUIT");
    session.send(b"q");

    let status = session.wait_for_exit(None);
    let completed = session.finish(status);
    assert!(
        completed.status.success(),
        "PTY output was: {}",
        completed.output.escape_debug()
    );
    completed.assert_terminal_restored();
    completed.assert_screen_lifecycle();
}

fn assert_queue_state(session: &mut PtySession, stage: usize) {
    for expected in [
        "CURRENT  Night Geometry [MORROW] • PLAYING",
        "QUEUE • 19 Tracks • Track 1/19",
        "SELECTED > TRACK     Night Geometry",
    ] {
        session.wait_for_screen_state(stage, expected);
    }
}
