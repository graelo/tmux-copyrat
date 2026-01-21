use std::collections::HashMap;
use std::fmt;

use clap::{Args, Parser, ValueEnum};

use super::basic;
use crate::{textbuf::alphabet, tmux, ui, Error, Result};

#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub enum MainConfig {
    /// Run tmux-copyrat.
    Run {
        #[command(flatten)]
        config_ext: ConfigExt,
    },
    /// Print the tmux plugin config file for initial configuration.
    Init,
}

/// Extended configuration for handling Tmux-specific configuration (options
/// and outputs). This is only used by `tmux-copyrat` and parsed from command
/// line.
#[derive(Args, Debug)]
pub struct ConfigExt {
    /// Don't read options from Tmux.
    ///
    /// By default, options formatted like `copyrat-*` are read from tmux.
    /// However, you should consider reading them from the config file (the
    /// default option) as this saves both a command call (about 10ms) and a
    /// Regex compilation.
    #[arg(short = 'n', long)]
    pub ignore_tmux_options: bool,

    /// Name of the copyrat temporary Tmux window.
    ///
    /// Copyrat is launched in a temporary window of that name. The only pane
    /// in this temp window gets swapped with the current active one for
    /// in-place searching, then swapped back and killed after we exit.
    #[arg(short = 'W', long, default_value = "[copyrat]")]
    pub window_name: String,

    /// Capture visible area or entire pane history.
    #[arg(
        value_enum,
        long,
        rename_all = "kebab-case",
        default_value = "visible-area"
    )]
    pub capture_region: CaptureRegion,

    /// Name of the copy-to-clipboard executable.
    ///
    /// If during execution, the output destination is set to be clipboard,
    /// then copyrat will pipe the selected text to this executable.
    /// On macOS, this is `pbcopy`, on Linux, this is `xclip`.
    #[arg(long, default_value = "pbcopy")]
    pub clipboard_exe: String,

    // Include fields from the basic config
    #[command(flatten)]
    pub basic_config: basic::Config,
}

impl ConfigExt {
    /// Finalize the ConfigExt by merging the tmux options.
    pub fn build(mut self) -> Result<ConfigExt> {
        if !self.ignore_tmux_options {
            let tmux_options: HashMap<String, String> = tmux::get_options("@copyrat-")?;

            // Override default values with those coming from tmux.
            let inner = &mut self.basic_config;

            for (name, value) in &tmux_options {
                match name.as_ref() {
                    // Note: @copyrat-capture-region is intentionally not read here.
                    // The capture region is determined by which keyswitch is used
                    // (prefix+t for visible-area, prefix+T for entire-history).
                    "@copyrat-alphabet" => {
                        inner.alphabet = alphabet::parse_alphabet(value)?;
                    }
                    "@copyrat-reverse" => {
                        inner.reverse = value.parse::<bool>()?;
                    }
                    "@copyrat-unique-hint" => {
                        inner.unique_hint = value.parse::<bool>()?;
                    }
                    "@copyrat-focus-wrap-around" => {
                        inner.focus_wrap_around = value.parse::<bool>()?
                    }

                    "@copyrat-text-fg" => inner.colors.text_fg = ui::colors::parse_color(value)?,
                    "@copyrat-text-bg" => inner.colors.text_bg = ui::colors::parse_color(value)?,
                    "@copyrat-span-fg" => inner.colors.span_fg = ui::colors::parse_color(value)?,
                    "@copyrat-span-bg" => inner.colors.span_bg = ui::colors::parse_color(value)?,
                    "@copyrat-focused-fg" => {
                        inner.colors.focused_fg = ui::colors::parse_color(value)?
                    }
                    "@copyrat-focused-bg" => {
                        inner.colors.focused_bg = ui::colors::parse_color(value)?
                    }
                    "@copyrat-hint-fg" => inner.colors.hint_fg = ui::colors::parse_color(value)?,
                    "@copyrat-hint-bg" => inner.colors.hint_bg = ui::colors::parse_color(value)?,

                    "@copyrat-hint-alignment" => {
                        let case_insensitive = true;
                        inner.hint_alignment = ui::HintAlignment::from_str(value, case_insensitive)
                            .map_err(Error::ExpectedEnumVariant)?
                    }
                    "@copyrat-hint-style" => {
                        let case_insensitive = true;
                        inner.hint_style_arg = Some(
                            basic::HintStyleArg::from_str(value, case_insensitive)
                                .map_err(Error::ExpectedEnumVariant)?,
                        )
                    }
                    "@copyrat-hint-surroundings" => {
                        inner.hint_surroundings = basic::try_parse_chars(value)?;
                    }
                    "@copyrat-default-output" => {
                        let case_insensitive = true;
                        inner.default_output =
                            OutputDestination::from_str(value, case_insensitive)
                                .map_err(Error::ExpectedEnumVariant)?
                    }

                    // Ignore unknown options.
                    _ => (),
                }
            }
        }

        Ok(self)
    }
}

/// Specifies which region of the terminal buffer to capture.
#[derive(Debug, Clone, ValueEnum, Parser)]
pub enum CaptureRegion {
    /// The entire history.
    // This will end up sending `-S - -E -` to `tmux capture-pane`.
    EntireHistory,
    /// The visible area.
    VisibleArea,
    ///// Region from start line to end line
    /////
    ///// This works as defined in tmux's docs (order does not matter).
    //Region(i32, i32),
}

/// Describes the type of buffer the selected should be copied to: either a
/// tmux buffer or the system clipboard.
#[derive(Clone, Debug, ValueEnum)]
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
