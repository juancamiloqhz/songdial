use std::{env, process::ExitCode};

mod terminal;

const HELP: &str = concat!(
    "Songdial — tune into the right music.\n",
    "\n",
    "Usage: songdial [OPTIONS]\n",
    "\n",
    "Options:\n",
    "      --no-motion  Disable motion treatments\n",
    "  -h, --help       Print help\n",
    "  -V, --version    Print version\n",
);

enum Command {
    Run { no_motion: bool },
    Help,
    Version,
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut no_motion = false;

    for argument in arguments {
        match argument.as_str() {
            "--no-motion" => no_motion = true,
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            _ => return Err(argument),
        }
    }

    Ok(Command::Run { no_motion })
}

fn main() -> ExitCode {
    match parse(env::args().skip(1)) {
        Ok(Command::Help) => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("songdial {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Command::Run { no_motion }) => {
            if let Err(error) = terminal::run(no_motion) {
                eprintln!("error: {error}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(argument) => {
            eprintln!("error: unexpected argument '{argument}'\n\nUsage: songdial [OPTIONS]");
            ExitCode::FAILURE
        }
    }
}
