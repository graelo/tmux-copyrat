use std::{
    io::{self, Read, Write},
    process::ExitCode,
};

use clap::Parser;

use copyrat::{config::basic, run, ui::Selection};

const EXIT_NO_SELECTION: u8 = 1;
const EXIT_IO_ERROR: u8 = 2;

fn main() -> ExitCode {
    try_main().unwrap_or_else(|error| {
        eprintln!("copyrat: {error}");
        ExitCode::from(EXIT_IO_ERROR)
    })
}

fn try_main() -> io::Result<ExitCode> {
    let opt = basic::Config::parse();

    // Copy the pane contents (piped in via stdin) into a buffer, and split lines.
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();
    handle.read_to_string(&mut buffer)?;
    let lines = buffer.split('\n').collect::<Vec<_>>();

    // Execute copyrat over the buffer (will take control over stdout).
    // This returns the selected span of text.
    let selection: Option<Selection> = run(&lines, &opt);

    // Early exit, signaling no selections were found.
    let Some(Selection { text, .. }) = selection else {
        return Ok(ExitCode::from(EXIT_NO_SELECTION));
    };
    writeln!(io::stdout().lock(), "{text}")?;
    Ok(ExitCode::SUCCESS)
}
