use clap::Clap;
use std::collections::HashMap;
use std::str::FromStr;

use super::basic;
use crate::comm::tmux;
use crate::error;

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct Config {
    /// Don't read options from Tmux.
    ///
    /// By default, options formatted like `copyrat-*` are read from tmux.
    /// However, you should consider reading them from the config file (the
    /// default option) as this saves both a command call (about 10ms) and a
    /// Regex compilation.
    #[clap(long)]
    pub ignore_options_from_tmux: bool,

    /// Name of the copyrat temporary window.
    ///
    /// Copyrat is launched in a temporary window of that name. The only pane
    /// in this temp window gets swapped with the current active one for
    /// in-place searching, then swapped back and killed after we exit.
    #[clap(long, default_value = "[copyrat]")]
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

impl Config {
    pub fn initialize() -> Result<Config, error::ParseError> {
        let mut config = Config::parse();

        if !config.ignore_options_from_tmux {
            let tmux_options: HashMap<String, String> = tmux::get_options("@copyrat-")?;

            // Override default values with those coming from tmux.
            config.merge_map(&tmux_options)?;
        }

        Ok(config)
    }
    /// Try parsing provided options, and update self with the valid values.
    /// Unknown options are simply ignored.
    pub fn merge_map(
        &mut self,
        options: &HashMap<String, String>,
    ) -> Result<(), error::ParseError> {
        for (name, value) in options {
            if let "@copyrat-capture" = name.as_ref() {
                self.capture_region = CaptureRegion::from_str(&value)?;
            }
        }

        // Pass the call to cli_options.
        self.basic_config.merge_map(options)?;

        Ok(())
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
