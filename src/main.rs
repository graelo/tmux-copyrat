use clap::Clap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, Read};

use copyrat::{run, CliOpt};

fn main() {
    let opt = CliOpt::parse();

    // Copy the pane contents (piped in via stdin) into a buffer, and split lines.
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();
    handle.read_to_string(&mut buffer).unwrap();

    // Execute copyrat over the buffer (will take control over stdout).
    // This returns the selected matche.
    let selection: Option<(String, bool)> = run(buffer, &opt);

    // Early exit, signaling no selections were found.
    if selection.is_none() {
        std::process::exit(1);
    }

    let (text, _) = selection.unwrap();

    // Write output to a target_path if provided, else print to original stdout.
    match opt.target_path {
        None => println!("{}", text),
        Some(target) => {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(target)
                .expect("Unable to open the target file");

            file.write_all(text.as_bytes()).unwrap();
        }
    }
}
