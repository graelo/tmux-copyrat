use clap::Clap;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use super::basic;
use crate::{
    error,
    textbuf::{alphabet, regexes},
    tmux, ui,
};

/// Extended configuration for handling Tmux-specific configuration (options
/// and outputs). This is only used by `tmux-copyrat` and parsed from command
/// line..
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct ConfigExt {
    /// Don't read options from Tmux.
    ///
    /// By default, options formatted like `copyrat-*` are read from tmux.
    /// However, you should consider reading them from the config file (the
    /// default option) as this saves both a command call (about 10ms) and a
    /// Regex compilation.
    #[clap(short = 'n', long)]
    pub ignore_tmux_options: bool,

    /// Name of the copyrat temporary Tmux window.
    ///
    /// Copyrat is launched in a temporary window of that name. The only pane
    /// in this temp window gets swapped with the current active one for
    /// in-place searching, then swapped back and killed after we exit.
    #[clap(short = 'W', long, default_value = "[copyrat]")]
    pub window_name: String,

    /// Capture visible area or entire pane history.
    #[clap(long, arg_enum, default_value = "visible-area")]
    pub capture_region: CaptureRegion,

    /// Name of the copy-to-clipboard executable.
    ///
    /// If during execution, the output destination is set to be clipboard,
    /// then copyrat will pipe the selected text to this executable.
    #[clap(long, default_value = "pbcopy")]
    pub clipboard_exe: String,

    // Include fields from the basic config
    #[clap(flatten)]
    pub basic_config: basic::Config,
}

impl ConfigExt {
    pub fn initialize() -> Result<ConfigExt, error::ParseError> {
        let mut config_ext = ConfigExt::parse();

        if !config_ext.ignore_tmux_options {
            let tmux_options: HashMap<String, String> = tmux::get_options("@copyrat-")?;

            // Override default values with those coming from tmux.
            let wrapped = &mut config_ext.basic_config;

            for (name, value) in &tmux_options {
                match name.as_ref() {
                    "@copyrat-capture" => {
                        config_ext.capture_region = CaptureRegion::from_str(&value)?
                    }
                    "@copyrat-alphabet" => {
                        wrapped.alphabet = alphabet::parse_alphabet(value)?;
                    }
                    "@copyrat-pattern-name" => {
                        wrapped.named_patterns = vec![regexes::parse_pattern_name(value)?]
                    }
                    "@copyrat-custom-pattern" => {
                        wrapped.custom_patterns = vec![String::from(value)]
                    }
                    "@copyrat-reverse" => {
                        wrapped.reverse = value.parse::<bool>()?;
                    }
                    "@copyrat-unique-hint" => {
                        wrapped.unique_hint = value.parse::<bool>()?;
                    }

                    "@copyrat-span-fg" => wrapped.colors.span_fg = ui::colors::parse_color(value)?,
                    "@copyrat-span-bg" => wrapped.colors.span_bg = ui::colors::parse_color(value)?,
                    "@copyrat-focused-fg" => {
                        wrapped.colors.focused_fg = ui::colors::parse_color(value)?
                    }
                    "@copyrat-focused-bg" => {
                        wrapped.colors.focused_bg = ui::colors::parse_color(value)?
                    }
                    "@copyrat-hint-fg" => wrapped.colors.hint_fg = ui::colors::parse_color(value)?,
                    "@copyrat-hint-bg" => wrapped.colors.hint_bg = ui::colors::parse_color(value)?,

                    "@copyrat-hint-alignment" => {
                        wrapped.hint_alignment = ui::HintAlignment::from_str(&value)?
                    }
                    "@copyrat-hint-style" => {
                        wrapped.hint_style = Some(basic::HintStyleArg::from_str(&value)?)
                    }

                    // Ignore unknown options.
                    _ => (),
                }
            }
        }

        Ok(config_ext)
    }
}

#[derive(Clap, Debug)]
pub enum CaptureRegion {
    /// The entire history.
    ///
    /// This will end up sending `-S - -E -` to `tmux capture-pane`.
    EntireHistory,
    /// The visible area.
    VisibleArea,
    ///// Region from start line to end line
    /////
    ///// This works as defined in tmux's docs (order does not matter).
    //Region(i32, i32),
}

impl FromStr for CaptureRegion {
    type Err = error::ParseError;

    fn from_str(s: &str) -> Result<Self, error::ParseError> {
        match s {
            "leading" => Ok(CaptureRegion::EntireHistory),
            "trailing" => Ok(CaptureRegion::VisibleArea),
            _ => Err(error::ParseError::ExpectedString(String::from(
                "entire-history or visible-area",
            ))),
        }
    }
}

/// Describes the type of buffer the selected should be copied to: either a
/// tmux buffer or the system clipboard.
#[derive(Clone)]
pub enum OutputDestination {
    /// The selection will be copied to the tmux buffer.
    Tmux,
    /// The selection will be copied to the system clipboard.
    Clipboard,
}

impl OutputDestination {
    /// Toggle between the variants of `OutputDestination`.
    pub fn toggle(&mut self) {
        match *self {
            Self::Tmux => *self = Self::Clipboard,
            Self::Clipboard => *self = Self::Tmux,
        }
    }
}

impl fmt::Display for OutputDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tmux => write!(f, "tmux buffer"),
            Self::Clipboard => write!(f, "clipboard"),
        }
    }
}
