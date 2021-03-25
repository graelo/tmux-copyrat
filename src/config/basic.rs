use clap::Clap;
use std::str::FromStr;

use crate::{
    error,
    textbuf::{alphabet, regexes},
    ui,
};

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct Config {
    /// Alphabet to draw hints from.
    ///
    /// Possible values are "{A}", "{A}-homerow", "{A}-left-hand",
    /// "{A}-right-hand", where "{A}" is one of "qwerty", "azerty", "qwertz"
    /// "dvorak", "colemak".
    ///
    /// # Examples
    ///
    /// "qwerty", "dvorak-homerow", "azerty-right-hand".
    #[clap(short = 'k', long, default_value = "dvorak",
                parse(try_from_str = alphabet::parse_alphabet))]
    pub alphabet: alphabet::Alphabet,

    /// Use all available regex patterns.
    #[clap(short = 'A', long = "--all-patterns")]
    pub use_all_patterns: bool,

    /// Pattern names to use ("email", ... see doc).
    #[clap(short = 'x', long = "--pattern-name", parse(try_from_str = regexes::parse_pattern_name))]
    pub named_patterns: Vec<regexes::NamedPattern>,

    /// Additional regex patterns ("foo*bar", etc).
    #[clap(short = 'X', long = "--custom-pattern")]
    pub custom_patterns: Vec<String>,

    /// Assign hints starting from the bottom of the screen.
    #[clap(short, long)]
    pub reverse: bool,

    /// Keep the same hint for identical spans.
    #[clap(short, long)]
    pub unique_hint: bool,

    /// Move focus back to first/last span.
    #[clap(short = 'w', long)]
    pub focus_wrap_around: bool,

    #[clap(flatten)]
    pub colors: ui::colors::UiColors,

    /// Align hint with its span.
    #[clap(long, arg_enum, default_value = "leading")]
    pub hint_alignment: ui::HintAlignment,

    /// Optional hint styling.
    ///
    /// Underline or surround the hint for increased visibility.
    /// If not provided, only the hint colors will be used.
    #[clap(short = 's', long, arg_enum)]
    pub hint_style: Option<HintStyleArg>,

    /// Chars surrounding each hint, used with `Surround` style.
    #[clap(long, default_value = "{}",
                parse(try_from_str = parse_chars))]
    pub hint_surroundings: (char, char),
}

/// Type introduced due to parsing limitation,
/// as we cannot directly parse into ui::HintStyle.
#[derive(Debug, Clap)]
pub enum HintStyleArg {
    Bold,
    Italic,
    Underline,
    Surround,
}

impl FromStr for HintStyleArg {
    type Err = error::ParseError;

    fn from_str(s: &str) -> Result<Self, error::ParseError> {
        match s {
            "bold" => Ok(HintStyleArg::Bold),
            "italic" => Ok(HintStyleArg::Italic),
            "underline" => Ok(HintStyleArg::Underline),
            "surrond" => Ok(HintStyleArg::Surround),
            _ => Err(error::ParseError::ExpectedString(String::from(
                "bold, italic, underline or surround",
            ))),
        }
    }
}

/// Try to parse a `&str` into a tuple of `char`s.
fn parse_chars(src: &str) -> Result<(char, char), error::ParseError> {
    if src.chars().count() != 2 {
        return Err(error::ParseError::ExpectedSurroundingPair);
    }

    let chars: Vec<char> = src.chars().collect();
    Ok((chars[0], chars[1]))
}
