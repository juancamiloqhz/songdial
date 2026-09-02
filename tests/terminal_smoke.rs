use std::{
    io::{Read, Write},
    sync::mpsc,
    time::{Duration, Instant},
};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

fn assert_home_and_restoration_after(quit_key: &[u8]) {
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
}

#[test]
fn real_binary_shows_home_quits_and_restores_the_terminal() {
    assert_home_and_restoration_after(b"q");
}

#[test]
fn control_c_exits_immediately_and_restores_the_terminal() {
    assert_home_and_restoration_after(b"\x03");
}
