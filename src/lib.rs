use clap::Clap;
use std::collections::HashMap;
use std::path;
use std::str::FromStr;

pub mod alphabets;
pub mod colors;
pub mod error;
pub mod model;
pub mod process;
pub mod regexes;
pub mod view;

/// Run copyrat on an input string `buffer`, configured by `Opt`.
///
/// # Note
///
/// Maybe the decision to take ownership of the buffer is a bit bold.
pub fn run(buffer: String, opt: &CliOpt) -> Option<(String, bool)> {
    let lines: Vec<&str> = buffer.split('\n').collect();

    let mut model = model::Model::new(
        &lines,
        &opt.alphabet,
        &opt.named_pattern,
        &opt.custom_regex,
        opt.reverse,
    );

    let hint_style = match &opt.hint_style {
        None => None,
        Some(style) => match style {
            HintStyleCli::Bold => Some(view::HintStyle::Bold),
            HintStyleCli::Italic => Some(view::HintStyle::Italic),
            HintStyleCli::Underline => Some(view::HintStyle::Underline),
            HintStyleCli::Surround => {
                let (open, close) = opt.hint_surroundings;
                Some(view::HintStyle::Surround(open, close))
            }
        },
    };

    let selection: Option<(String, bool)> = {
        let mut viewbox = view::View::new(
            &mut model,
            opt.unique_hint,
            &opt.hint_alignment,
            &opt.colors,
            hint_style,
        );

        viewbox.present()
    };

    selection
}

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct CliOpt {
    /// Alphabet to draw hints from.
    ///
    /// Possible values are "{A}", "{A}-homerow", "{A}-left-hand",
    /// "{A}-right-hand", where "{A}" is one of "qwerty", "azerty", "qwertz"
    /// "dvorak", "colemak".
    ///
    /// # Examples
    ///
    /// "qwerty", "dvorak-homerow", "azerty-right-hand".
    #[clap(short = "k", long, default_value = "dvorak",
                parse(try_from_str = alphabets::parse_alphabet))]
    alphabet: alphabets::Alphabet,

    /// Pattern names to use (all if not specified).
    #[clap(short = "x", long = "--pattern-name", parse(try_from_str = regexes::parse_pattern_name))]
    named_pattern: Vec<regexes::NamedPattern>,

    /// Additional regex patterns.
    #[clap(short = "X", long)]
    custom_regex: Vec<String>,

    /// Assign hints starting from the bottom of the screen.
    #[clap(short, long)]
    reverse: bool,

    /// Keep the same hint for identical matches.
    #[clap(short, long)]
    unique_hint: bool,

    #[clap(flatten)]
    colors: view::ViewColors,

    /// Align hint with its match.
    #[clap(short = "a", long, arg_enum, default_value = "leading")]
    hint_alignment: view::HintAlignment,

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

    /// Optional target path where to store the selected matches.
    #[clap(short = "o", long = "output", parse(from_os_str))]
    pub target_path: Option<path::PathBuf>,

    /// Describes if the uppercased marker should be added to the output,
    /// indicating if hint key was uppercased. This is only used by
    /// tmux-copyrat, so it is hidden (skipped) from the CLI.
    #[clap(skip)]
    pub uppercased_marker: bool,
}

/// Type introduced due to parsing limitation,
/// as we cannot directly parse into view::HintStyle.
#[derive(Debug, Clap)]
enum HintStyleCli {
    Bold,
    Italic,
    Underline,
    Surround,
}

impl FromStr for HintStyleCli {
    type Err = error::ParseError;

    fn from_str(s: &str) -> Result<Self, error::ParseError> {
        match s {
            "leading" => Ok(HintStyleCli::Underline),
            "trailing" => Ok(HintStyleCli::Surround),
            _ => Err(error::ParseError::ExpectedString(String::from(
                "underline or surround",
            ))),
        }
    }
}

/// Try to parse a `&str` into a tuple of `char`s.
fn parse_chars(src: &str) -> Result<(char, char), error::ParseError> {
    if src.len() != 2 {
        return Err(error::ParseError::ExpectedSurroundingPair);
    }

    let chars: Vec<char> = src.chars().collect();
    Ok((chars[0], chars[1]))
}

impl CliOpt {
    /// Try parsing provided options, and update self with the valid values.
    pub fn merge_map(
        &mut self,
        options: &HashMap<String, String>,
    ) -> Result<(), error::ParseError> {
        for (name, value) in options {
            match name.as_ref() {
                "@copyrat-alphabet" => {
                    self.alphabet = alphabets::parse_alphabet(value)?;
                }
                "@copyrat-regex-id" => (), // TODO
                "@copyrat-custom-regex" => self.custom_regex = vec![String::from(value)],
                "@copyrat-reverse" => {
                    self.reverse = value.parse::<bool>()?;
                }
                "@copyrat-unique-hint" => {
                    self.unique_hint = value.parse::<bool>()?;
                }

                "@copyrat-match-fg" => self.colors.match_fg = colors::parse_color(value)?,
                "@copyrat-match-bg" => self.colors.match_bg = colors::parse_color(value)?,
                "@copyrat-focused-fg" => self.colors.focused_fg = colors::parse_color(value)?,
                "@copyrat-focused-bg" => self.colors.focused_bg = colors::parse_color(value)?,
                "@copyrat-hint-fg" => self.colors.hint_fg = colors::parse_color(value)?,
                "@copyrat-hint-bg" => self.colors.hint_bg = colors::parse_color(value)?,

                "@copyrat-hint-alignment" => {
                    self.hint_alignment = view::HintAlignment::from_str(&value)?
                }
                "@copyrat-hint-style" => self.hint_style = Some(HintStyleCli::from_str(&value)?),

                // Ignore unknown options.
                _ => (),
            }
        }

        Ok(())
    }
}
