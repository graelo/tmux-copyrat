use clap::Clap;
use std::io::{self, Read};

use copyrat::{run, Opt};

fn main() {
    let opt = Opt::parse();

    // Copy the pane contents (piped in via stdin) into a buffer, and split lines.
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();
    handle.read_to_string(&mut buffer).unwrap();

    run(buffer, opt);
}
