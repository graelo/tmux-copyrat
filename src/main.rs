use clap::Clap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, Read};

use copyrat::{run, Opt};

fn main() {
    let opt = Opt::parse();

    // Copy the pane contents (piped in via stdin) into a buffer, and split lines.
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();
    handle.read_to_string(&mut buffer).unwrap();

    // Execute copyrat over the buffer (will take control over stdout).
    // This returns the selected matches.
    let output: String = run(buffer, &opt);

    // Write output to a target_path if provided, else print to original stdout.
    match opt.target_path {
        None => println!("{}", output),
        Some(target) => {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(target)
                .expect("Unable to open the target file");

            file.write(output.as_bytes()).unwrap();
        }
    }
}
