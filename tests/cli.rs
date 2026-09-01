use std::process::{Command, Output};

fn songdial(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_songdial"))
        .args(arguments)
        .output()
        .expect("songdial binary should run")
}

#[test]
fn help_describes_the_complete_command_line_surface() {
    let output = songdial(&["--help"]);

    assert_eq!(
        (
            output.status.success(),
            String::from_utf8(output.stdout).unwrap()
        ),
        (
            true,
            concat!(
                "Songdial — tune into the right music.\n",
                "\n",
                "Usage: songdial [OPTIONS]\n",
                "\n",
                "Options:\n",
                "      --no-motion  Disable motion treatments\n",
                "  -h, --help       Print help\n",
                "  -V, --version    Print version\n",
            )
            .to_owned()
        )
    );
}

#[test]
fn version_reports_the_package_version() {
    let output = songdial(&["--version"]);

    assert_eq!(
        (
            output.status.success(),
            String::from_utf8(output.stdout).unwrap()
        ),
        (true, format!("songdial {}\n", env!("CARGO_PKG_VERSION")))
    );
}

#[test]
fn undeclared_options_fail_without_entering_the_terminal() {
    let output = songdial(&["--theme"]);

    assert_eq!(
        (
            output.status.success(),
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ),
        (
            false,
            String::new(),
            "error: unexpected argument '--theme'\n\nUsage: songdial [OPTIONS]\n".to_owned(),
        )
    );
}
