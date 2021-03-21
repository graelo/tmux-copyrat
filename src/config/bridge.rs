use clap::Clap;
use std::collections::HashMap;
use std::str::FromStr;

use super::CliOpt;
use crate::comm::tmux;
use crate::error;

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct BridgeOpt {
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
    pub capture_region: tmux::CaptureRegion,

    /// Name of the copy-to-clipboard executable.
    ///
    /// If during execution, the output destination is set to be clipboard,
    /// then copyrat will pipe the selected text to this executable.
    #[clap(long, default_value = "pbcopy")]
    pub clipboard_exe: String,

    // Include CLI Options
    #[clap(flatten)]
    pub cli_options: CliOpt,
}

impl BridgeOpt {
    /// Try parsing provided options, and update self with the valid values.
    /// Unknown options are simply ignored.
    pub fn merge_map(
        &mut self,
        options: &HashMap<String, String>,
    ) -> Result<(), error::ParseError> {
        for (name, value) in options {
            if let "@copyrat-capture" = name.as_ref() {
                self.capture_region = tmux::CaptureRegion::from_str(&value)?;
            }
        }

        // Pass the call to cli_options.
        self.cli_options.merge_map(options)?;

        Ok(())
    }
}
