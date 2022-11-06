use std::fmt::Display;

use clap::{ArgAction, Parser, ValueEnum};

use crate::{
    textbuf::{alphabet, regexes},
    ui, Error, Result,
};

/// Main configuration, parsed from command line.
#[derive(Parser, Debug)]
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
    #[arg(
        short = 'k',
        long,
        default_value = "dvorak",
        value_parser(alphabet::parse_alphabet)
    )]
    pub alphabet: alphabet::Alphabet,

    /// Use all available regex patterns.
    #[arg(short = 'A', long = "all-patterns")]
    pub use_all_patterns: bool,

    /// Pattern names to use ("email", ... see doc).
    #[arg(
        short = 'x',
        long = "pattern-name",
        value_parser(regexes::parse_pattern_name)
    )]
    pub named_patterns: Vec<regexes::NamedPattern>,

    /// Additional regex patterns ("(foo.*)bar", etc). Must have a capture
    /// group.
    #[arg(short = 'X', long)]
    pub custom_patterns: Vec<String>,

    /// Assign hints starting from the bottom of the screen.
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub reverse: bool,

    /// Keep the same hint for identical spans.
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub unique_hint: bool,

    /// Move focus back to first/last span.
    #[arg(short = 'w', long, action = ArgAction::SetTrue)]
    pub focus_wrap_around: bool,

    #[command(flatten)]
    pub colors: ui::colors::UiColors,

    /// Align hint with its span.
    #[arg(long, value_enum, default_value_t = ui::HintAlignment::Leading)]
    pub hint_alignment: ui::HintAlignment,

    /// Optional hint styling.
    ///
    /// Underline or surround the hint for increased visibility.
    /// If not provided, only the hint colors will be used.
    #[arg(short = 's', long = "hint-style", rename_all = "lowercase", value_enum)]
    pub hint_style_arg: Option<HintStyleArg>,

    /// Chars surrounding each hint, used with `Surround` style.
    #[clap(
        long,
        // default_value_t = HintSurroundingsArg{open: '{', close: '}'},
        default_value = "{}",
        value_parser(try_parse_chars)
    )]
    pub hint_surroundings: HintSurroundingsArg,
}

/// Type introduced due to parsing limitation,
/// as we cannot directly parse tuples into ui::HintStyle.
#[derive(Debug, Clone, ValueEnum)]
pub enum HintStyleArg {
    Bold,
    Italic,
    Underline,
    Surround,
}

#[derive(Debug, Clone)]
pub struct HintSurroundingsArg {
    pub open: char,
    pub close: char,
}

impl Display for HintSurroundingsArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.open, self.close)
    }
}

/// Try to parse a `&str` into a tuple of `char`s.
fn try_parse_chars(src: &str) -> Result<HintSurroundingsArg> {
    if src.chars().count() != 2 {
        return Err(Error::ExpectedSurroundingPair);
    }

    let chars: Vec<char> = src.chars().collect();
    Ok(HintSurroundingsArg {
        open: chars[0],
        close: chars[1],
    })
}

impl Config {
    pub fn hint_style(&self) -> Option<ui::HintStyle> {
        match &self.hint_style_arg {
            None => None,
            Some(style) => match style {
                HintStyleArg::Bold => Some(ui::HintStyle::Bold),
                HintStyleArg::Italic => Some(ui::HintStyle::Italic),
                HintStyleArg::Underline => Some(ui::HintStyle::Underline),
                HintStyleArg::Surround => {
                    let HintSurroundingsArg { open, close } = self.hint_surroundings;
                    Some(ui::HintStyle::Surround(open, close))
                }
            },
        }
    }
}
