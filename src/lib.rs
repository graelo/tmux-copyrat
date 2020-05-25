use clap::Clap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path;

pub mod alphabets;
pub mod colors;
pub mod error;
pub mod state;
pub mod view;

/// Run copyrat on an input string `buffer`, configured by `Opt`.
///
/// # Note
///
/// Maybe the decision to move ownership is a bit bold.
pub fn run(buffer: String, opt: Opt) {
    let lines: Vec<&str> = buffer.split('\n').collect();

    let mut state = state::State::new(&lines, &opt.alphabet, &opt.custom_regex);

    let hint_style = match opt.hint_style {
        None => None,
        Some(style) => match style {
            HintStyleCli::Underline => Some(view::HintStyle::Underline),
            HintStyleCli::Surround => {
                let (open, close) = opt.hint_surroundings;
                Some(view::HintStyle::Surround(open, close))
            }
        },
    };
    let uppercased_marker = opt.uppercased_marker;

    let selections: Vec<(String, bool)> = {
        let mut viewbox = view::View::new(
            &mut state,
            opt.multi_selection,
            opt.reverse,
            opt.unique,
            opt.hint_alignment,
            &opt.colors,
            hint_style,
        );

        viewbox.present()
    };

    // Early exit, signaling tmux we had no selections.
    if selections.is_empty() {
        std::process::exit(1);
    }

    let output: String = if uppercased_marker {
        selections
            .iter()
            .map(|(text, uppercased)| format!("{}:{}", *uppercased, text))
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        selections
            .iter()
            .map(|(text, _)| text.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    };

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

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct Opt {
    /// Alphabet to draw hints from.
    ///
    /// Possible values are "{A}", "{A}-homerow", "{A}-left-hand",
    /// "{A}-right-hand", where "{A}" is one of "qwerty", "azerty", "qwertz",
    /// "dvorak", "colemak". Examples: "qwerty", "dvorak-homerow".
    #[clap(short = "k", long, default_value = "qwerty",
                parse(try_from_str = alphabets::parse_alphabet))]
    alphabet: alphabets::Alphabet,

    /// Enable multi-selection.
    #[clap(short, long)]
    multi_selection: bool,

    #[clap(flatten)]
    colors: view::ViewColors,

    /// Reverse the order for assigned hints.
    #[clap(short, long)]
    reverse: bool,

    /// Keep the same hint for identical matches.
    #[clap(short, long)]
    unique: bool,

    /// Align hint with its match.
    #[clap(short = "a", long, arg_enum, default_value = "Leading")]
    hint_alignment: view::HintAlignment,

    /// Additional regex patterns.
    #[clap(short = "c", long)]
    custom_regex: Vec<String>,

    /// Optional hint styling.
    ///
    /// Underline or surround the hint for increased visibility.
    /// If not provided, only the hint colors will be used.
    #[clap(short = "s", long, arg_enum)]
    hint_style: Option<HintStyleCli>,

    /// Chars surrounding each hint, used with `Surround` style.
    #[clap(long, default_value = "{}",
                parse(try_from_str = parse_chars))]
    hint_surroundings: (char, char),

    /// Target path where to store the selected matches.
    #[clap(short = "o", long = "output", parse(from_os_str))]
    target_path: Option<path::PathBuf>,

    /// Describes if the uppercased marker should be added to the output,
    /// indicating if hint key was uppercased. This is only used by
    /// tmux-copyrat, so it is skipped from clap configuration.
    #[clap(skip)]
    uppercased_marker: bool,
}

/// Type introduced due to parsing limitation,
/// as we cannot directly parse into view::HintStyle.
#[derive(Debug, Clap)]
enum HintStyleCli {
    Underline,
    Surround,
}

fn parse_chars(src: &str) -> Result<(char, char), error::ParseError> {
    if src.len() != 2 {
        return Err(error::ParseError::ExpectedSurroundingPair);
    }

    let chars: Vec<char> = src.chars().collect();
    Ok((chars[0], chars[1]))
}
