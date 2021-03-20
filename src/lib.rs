pub mod comm;
pub mod error;
pub mod output_destination;
pub mod textbuf;
pub mod ui;

use clap::Clap;
use std::collections::HashMap;
use std::path;
use std::str::FromStr;

use crate::textbuf::alphabet;
use crate::textbuf::regexes;

/// Run copyrat on an input string `buffer`, configured by `Opt`.
///
/// # Note
///
/// Maybe the decision to take ownership of the buffer is a bit bold.
pub fn run(buffer: String, opt: &CliOpt) -> Option<ui::Selection> {
    let mut model = textbuf::Model::new(
        &buffer,
        &opt.alphabet,
        opt.use_all_patterns,
        &opt.named_patterns,
        &opt.custom_patterns,
        opt.reverse,
    );

    let hint_style = match &opt.hint_style {
        None => None,
        Some(style) => match style {
            HintStyleCli::Bold => Some(ui::HintStyle::Bold),
            HintStyleCli::Italic => Some(ui::HintStyle::Italic),
            HintStyleCli::Underline => Some(ui::HintStyle::Underline),
            HintStyleCli::Surround => {
                let (open, close) = opt.hint_surroundings;
                Some(ui::HintStyle::Surround(open, close))
            }
        },
    };

    let default_output_destination = output_destination::OutputDestination::Tmux;

    let selection: Option<ui::Selection> = {
        let mut ui = ui::ViewController::new(
            &mut model,
            opt.unique_hint,
            opt.focus_wrap_around,
            default_output_destination,
            &opt.colors,
            &opt.hint_alignment,
            hint_style,
        );

        ui.present()
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
    #[clap(short = 'k', long, default_value = "dvorak",
                parse(try_from_str = alphabet::parse_alphabet))]
    alphabet: alphabet::Alphabet,

    /// Use all available regex patterns.
    #[clap(short = 'A', long = "--all-patterns")]
    use_all_patterns: bool,

    /// Pattern names to use ("email", ... see doc).
    #[clap(short = 'x', long = "--pattern-name", parse(try_from_str = regexes::parse_pattern_name))]
    named_patterns: Vec<regexes::NamedPattern>,

    /// Additional regex patterns ("foo*bar", etc).
    #[clap(short = 'X', long = "--custom-pattern")]
    custom_patterns: Vec<String>,

    /// Assign hints starting from the bottom of the screen.
    #[clap(short, long)]
    reverse: bool,

    /// Keep the same hint for identical matches.
    #[clap(short, long)]
    unique_hint: bool,

    #[clap(flatten)]
    colors: ui::colors::UiColors,

    /// Align hint with its match.
    #[clap(long, arg_enum, default_value = "leading")]
    hint_alignment: ui::HintAlignment,

    /// Move focus back to first/last match.
    #[clap(long)]
    focus_wrap_around: bool,

    /// Optional hint styling.
    ///
    /// Underline or surround the hint for increased visibility.
    /// If not provided, only the hint colors will be used.
    #[clap(short = 's', long, arg_enum)]
    hint_style: Option<HintStyleCli>,

    /// Chars surrounding each hint, used with `Surround` style.
    #[clap(long, default_value = "{}",
                parse(try_from_str = parse_chars))]
    hint_surroundings: (char, char),

    /// Optional target path where to store the selected matches.
    #[clap(short = 'o', long = "output", parse(from_os_str))]
    pub target_path: Option<path::PathBuf>,

    /// Describes if the uppercased marker should be added to the output,
    /// indicating if hint key was uppercased. This is only used by
    /// tmux-copyrat, so it is hidden (skipped) from the CLI.
    #[clap(skip)]
    pub uppercased_marker: bool,
}

/// Type introduced due to parsing limitation,
/// as we cannot directly parse into ui::HintStyle.
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
                    self.alphabet = alphabet::parse_alphabet(value)?;
                }
                "@copyrat-pattern-name" => {
                    self.named_patterns = vec![regexes::parse_pattern_name(value)?]
                }
                "@copyrat-custom-pattern" => self.custom_patterns = vec![String::from(value)],
                "@copyrat-reverse" => {
                    self.reverse = value.parse::<bool>()?;
                }
                "@copyrat-unique-hint" => {
                    self.unique_hint = value.parse::<bool>()?;
                }

                "@copyrat-match-fg" => self.colors.match_fg = ui::colors::parse_color(value)?,
                "@copyrat-match-bg" => self.colors.match_bg = ui::colors::parse_color(value)?,
                "@copyrat-focused-fg" => self.colors.focused_fg = ui::colors::parse_color(value)?,
                "@copyrat-focused-bg" => self.colors.focused_bg = ui::colors::parse_color(value)?,
                "@copyrat-hint-fg" => self.colors.hint_fg = ui::colors::parse_color(value)?,
                "@copyrat-hint-bg" => self.colors.hint_bg = ui::colors::parse_color(value)?,

                "@copyrat-hint-alignment" => {
                    self.hint_alignment = ui::HintAlignment::from_str(&value)?
                }
                "@copyrat-hint-style" => self.hint_style = Some(HintStyleCli::from_str(&value)?),

                // Ignore unknown options.
                _ => (),
            }
        }

        Ok(())
    }
}
