use std::{
    io::{Read, Write},
    sync::mpsc,
    time::{Duration, Instant},
};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

fn smoke_binary() -> std::ffi::OsString {
    std::env::var_os("SONGDIAL_SMOKE_BINARY")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_songdial").into())
}

fn assert_home_and_restoration_after(quit_key: &[u8], no_color: bool) {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("test PTY should open");
    let terminal_before = format!("{:?}", pair.master.get_termios());

    let mut command = CommandBuilder::new(smoke_binary());
    command.arg("--no-motion");
    command.env("TERM", "xterm-256color");
    if no_color {
        command.env("NO_COLOR", "1");
    }
    let mut child = pair
        .slave
        .spawn_command(command)
        .expect("songdial should start in a PTY");
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .expect("PTY output should be readable");
    let (output_sender, output_receiver) = mpsc::channel();
    let reader_thread = std::thread::spawn(move || {
        let mut chunk = [0_u8; 4096];
        while let Ok(read) = reader.read(&mut chunk) {
            if read == 0 {
                break;
            }
            if output_sender.send(chunk[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    let mut writer = pair
        .master
        .take_writer()
        .expect("PTY input should be writable");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut output = Vec::new();
    let mut saw_home = false;
    while Instant::now() < deadline {
        if let Ok(chunk) = output_receiver.recv_timeout(Duration::from_millis(25)) {
            output.extend(chunk);
            saw_home = String::from_utf8_lossy(&output).contains("Mood & activity");
            if saw_home {
                break;
            }
        }
        if child
            .try_wait()
            .expect("child status should be readable")
            .is_some()
        {
            break;
        }
    }

    if saw_home {
        writer.write_all(quit_key).expect("quit key should be sent");
        writer.flush().expect("quit key should be flushed");
    }

    let exit_deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            break status;
        }
        assert!(
            Instant::now() < exit_deadline,
            "songdial did not exit after the quit key"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(writer);

    while let Ok(chunk) = output_receiver.recv_timeout(Duration::from_millis(25)) {
        output.extend(chunk);
    }
    reader_thread.join().expect("PTY reader should finish");

    let terminal_after = format!("{:?}", pair.master.get_termios());
    let output = String::from_utf8_lossy(&output);

    assert_eq!(
        (
            saw_home,
            status.success(),
            output.contains("\u{1b}[?1049h"),
            output.contains("\u{1b}[?25l"),
            output.contains("\u{1b}[?25h"),
            output.contains("\u{1b}[?1049l"),
            terminal_after,
        ),
        (true, true, true, true, true, true, terminal_before),
        "PTY output was: {}",
        output.escape_debug()
    );
    if no_color {
        assert!(
            !output.contains("38;2;"),
            "PTY output was: {}",
            output.escape_debug()
        );
        assert!(
            !output.contains("48;2;"),
            "PTY output was: {}",
            output.escape_debug()
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
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("test PTY should open");
    let terminal_before = format!("{:?}", pair.master.get_termios());

    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_songdial"));
    command.arg("--no-motion");
    command.env("TERM", "xterm-256color");
    command.env("SONGDIAL_TEST_FAIL_STARTUP_AFTER_RAW_MODE", "1");
    let mut child = pair
        .slave
        .spawn_command(command)
        .expect("songdial should start in a PTY");
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .expect("PTY output should be readable");
    let reader_thread = std::thread::spawn(move || {
        let mut output = Vec::new();
        reader.read_to_end(&mut output).map(|_| output)
    });

    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().expect("hung child should be stopped");
            let status = child.wait().expect("stopped child should be reaped");
            break status;
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    let output = reader_thread
        .join()
        .expect("PTY reader should finish")
        .expect("PTY output should be readable");
    let terminal_after = format!("{:?}", pair.master.get_termios());
    let output = String::from_utf8_lossy(&output);

    assert_eq!(terminal_after, terminal_before);
    assert!(
        !status.success(),
        "PTY output was: {}",
        output.escape_debug()
    );
    assert!(
        output.contains("error: injected startup failure after raw mode"),
        "PTY output was: {}",
        output.escape_debug()
    );
}

#[test]
fn handled_runtime_failure_restores_the_terminal() {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("test PTY should open");
    let terminal_before = format!("{:?}", pair.master.get_termios());

    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_songdial"));
    command.arg("--no-motion");
    command.env("TERM", "xterm-256color");
    command.env("SONGDIAL_TEST_FAIL_RUNTIME_AFTER_DRAW", "1");
    let mut child = pair
        .slave
        .spawn_command(command)
        .expect("songdial should start in a PTY");
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .expect("PTY output should be readable");
    let (output_sender, output_receiver) = mpsc::channel();
    let reader_thread = std::thread::spawn(move || {
        let mut chunk = [0_u8; 4096];
        while let Ok(read) = reader.read(&mut chunk) {
            if read == 0 {
                break;
            }
            if output_sender.send(chunk[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    let mut writer = pair
        .master
        .take_writer()
        .expect("PTY input should be writable");

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut output = Vec::new();
    let status = loop {
        while let Ok(chunk) = output_receiver.try_recv() {
            output.extend(chunk);
        }
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            break status;
        }
        if Instant::now() >= deadline {
            writer
                .write_all(b"q")
                .expect("fallback quit key should be sent");
            writer.flush().expect("fallback quit key should be flushed");
            break child.wait().expect("child should exit after fallback quit");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(writer);

    while let Ok(chunk) = output_receiver.recv_timeout(Duration::from_millis(25)) {
        output.extend(chunk);
    }
    reader_thread.join().expect("PTY reader should finish");

    let terminal_after = format!("{:?}", pair.master.get_termios());
    let output = String::from_utf8_lossy(&output);

    assert_eq!(terminal_after, terminal_before);
    assert!(
        !status.success(),
        "PTY output was: {}",
        output.escape_debug()
    );
    assert!(
        output.contains("Mood & activity"),
        "PTY output was: {}",
        output.escape_debug()
    );
    assert!(
        output.contains("error: injected runtime failure after draw"),
        "PTY output was: {}",
        output.escape_debug()
    );
    assert!(output.contains("\u{1b}[?1049h"));
    assert!(output.contains("\u{1b}[?25l"));
    assert!(output.contains("\u{1b}[?25h"));
    assert!(output.contains("\u{1b}[?1049l"));
}

fn finish_output_stage(output_receiver: &mpsc::Receiver<Vec<u8>>, output: &mut Vec<u8>) -> usize {
    while let Ok(chunk) = output_receiver.try_recv() {
        output.extend(chunk);
    }
    output.len()
}

fn wait_for_screen_state(
    output_receiver: &mpsc::Receiver<Vec<u8>>,
    output: &mut Vec<u8>,
    stage_start: usize,
    expected: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(chunk) = output_receiver.recv_timeout(Duration::from_millis(25)) {
            output.extend(chunk);
            if String::from_utf8_lossy(&output[stage_start..]).contains(expected) {
                return;
            }
        }
    }

    panic!(
        "did not observe {expected:?}; PTY output was: {}",
        String::from_utf8_lossy(&output[stage_start..]).escape_debug()
    );
}

#[test]
fn packaged_binary_completes_a_meaningful_smoke_journey_and_restores_the_terminal() {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("test PTY should open");
    let terminal_before = format!("{:?}", pair.master.get_termios());

    let mut command = CommandBuilder::new(smoke_binary());
    command.arg("--no-motion");
    command.env_clear();
    command.env("TERM", "xterm-256color");
    let mut child = pair
        .slave
        .spawn_command(command)
        .expect("packaged songdial should start in a PTY");
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .expect("PTY output should be readable");
    let (output_sender, output_receiver) = mpsc::channel();
    let reader_thread = std::thread::spawn(move || {
        let mut chunk = [0_u8; 4096];
        while let Ok(read) = reader.read(&mut chunk) {
            if read == 0 {
                break;
            }
            if output_sender.send(chunk[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    let mut writer = pair
        .master
        .take_writer()
        .expect("PTY input should be writable");
    let mut output = Vec::new();

    wait_for_screen_state(&output_receiver, &mut output, 0, "Mood & activity");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer.write_all(b"\r").expect("open key should be sent");
    writer.flush().expect("open key should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "Deep Work");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer.write_all(b"/").expect("search key should be sent");
    writer.flush().expect("search key should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "Query >");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer
        .write_all(b"Night")
        .expect("search query should be sent");
    writer.flush().expect("search query should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "Night Geometry");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer
        .write_all(b"\x1b")
        .expect("search focus should close");
    writer.flush().expect("back key should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "edit query");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer.write_all(b"\x1b").expect("search should close");
    writer.flush().expect("back key should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "Deep Work");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer.write_all(b"\x1b").expect("browse should close");
    writer.flush().expect("back key should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "Mood & activity");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer
        .write_all(b"jj\r")
        .expect("playlist navigation should be sent");
    writer
        .flush()
        .expect("playlist navigation should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "MY PLAYLISTS");

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer.write_all(b"p").expect("play key should be sent");
    writer.flush().expect("play key should be flushed");
    wait_for_screen_state(
        &output_receiver,
        &mut output,
        stage,
        "Night Geometry [MORROW] • PLAYING",
    );

    let stage = finish_output_stage(&output_receiver, &mut output);
    writer.write_all(b"n").expect("queue key should be sent");
    writer.flush().expect("queue key should be flushed");
    wait_for_screen_state(&output_receiver, &mut output, stage, "CURRENT ");

    let stage = finish_output_stage(&output_receiver, &mut output);
    pair.master
        .resize(PtySize {
            rows: 23,
            cols: 79,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("PTY should resize below the supported minimum");
    wait_for_screen_state(&output_receiver, &mut output, stage, "Required 80×24 cells");

    let stage = finish_output_stage(&output_receiver, &mut output);
    pair.master
        .resize(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("PTY should resize back to the supported minimum");
    wait_for_screen_state(&output_receiver, &mut output, stage, "CURRENT ");

    writer.write_all(b"q").expect("quit key should be sent");
    writer.flush().expect("quit key should be flushed");
    let exit_deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            break status;
        }
        assert!(
            Instant::now() < exit_deadline,
            "songdial did not exit after the smoke journey"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(writer);

    while let Ok(chunk) = output_receiver.recv_timeout(Duration::from_millis(25)) {
        output.extend(chunk);
    }
    reader_thread.join().expect("PTY reader should finish");

    let terminal_after = format!("{:?}", pair.master.get_termios());
    let output = String::from_utf8_lossy(&output);
    assert!(
        status.success(),
        "PTY output was: {}",
        output.escape_debug()
    );
    assert_eq!(terminal_after, terminal_before);
    assert!(output.contains("\u{1b}[?25h"));
    assert!(output.contains("\u{1b}[?1049l"));
}
