use clap::Parser;
use std::io::{self, Read};

use copyrat::{config::basic, run, ui::Selection};

fn main() {
    let opt = basic::Config::parse();

    // Copy the pane contents (piped in via stdin) into a buffer, and split lines.
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();
    handle.read_to_string(&mut buffer).unwrap();
    let lines = buffer.split('\n').collect::<Vec<_>>();

    // Execute copyrat over the buffer (will take control over stdout).
    // This returns the selected span of text.
    let selection: Option<Selection> = run(&lines, &opt);

    // Early exit, signaling no selections were found.
    if selection.is_none() {
        std::process::exit(1);
    }

    let Selection { text, .. } = selection.unwrap();
    println!("{}", text);
}
