use clap::Clap;
use std::collections::HashMap;
use std::str::FromStr;

use copyrat::{error, process, CliOpt};

mod tmux;

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct BridgeOpt {
    ///// Command to execute on selection.
    //#[clap(long, default_value = "tmux set-buffer {}")]
    //command: String,

    ///// Command to execute on uppercased selection.
    /////
    ///// This defaults to pasting in the original pane.
    //#[clap(
    //    long,
    //    default_value = "tmux set-buffer {} && tmux paste-buffer -t '#{active_pane}"
    //)]
    //alt_command: String,
    /// Don't read options from Tmux.
    ///
    /// By default, options formatted like `copyrat-*` are read from tmux.
    /// However, you should consider reading them from the config file (the
    /// default option) as this saves both a command call (about 10ms) and a
    /// Regex compilation.
    #[clap(long)]
    ignore_options_from_tmux: bool,

    /// Name of the copyrat temporary window.
    ///
    /// Copyrat is launched in a temporary window of that name. The only pane
    /// in this temp window gets swapped with the current active one for
    /// in-place searching, then swapped back and killed after we exit.
    #[clap(long, default_value = "[copyrat]")]
    window_name: String,

    /// Capture visible area or entire pane history.
    #[clap(long, arg_enum, default_value = "visible-area")]
    capture_region: tmux::CaptureRegion,

    // Include CLI Options
    #[clap(flatten)]
    cli_options: CliOpt,
}

impl BridgeOpt {
    /// Try parsing provided options, and update self with the valid values.
    pub fn merge_map(
        &mut self,
        options: &HashMap<String, String>,
    ) -> Result<(), error::ParseError> {
        for (name, value) in options {
            match name.as_ref() {
                // "@copyrat-command" => {
                //     self.command = String::from(value);
                // }
                // "@copyrat-alt-command" => {
                //     self.alt_command = String::from(value);
                // }
                "@copyrat-capture" => {
                    self.capture_region = tmux::CaptureRegion::from_str(&value)?;
                }

                // Ignore unknown options.
                _ => (),
            }
        }

        // Pass the call to cli_options.
        self.cli_options.merge_map(options)?;

        Ok(())
    }
}

///
fn main() -> Result<(), error::ParseError> {
    let mut opt = BridgeOpt::parse();

    if !opt.ignore_options_from_tmux {
        let tmux_options: HashMap<String, String> = tmux::get_options("@copyrat-")?;

        // Override default values with those coming from tmux.
        opt.merge_map(&tmux_options)?;
    }

    let panes: Vec<tmux::Pane> = tmux::list_panes()?;

    let active_pane = panes
        .into_iter()
        .find(|p| p.is_active)
        .expect("Exactly one tmux pane should be active in the current window.");

    let buffer = tmux::capture_pane(&active_pane, &opt.capture_region)?;

    // We have to dance a little with Panes, because this process i/o streams
    // are connected to the pane in the window newly created for us, instead
    // of the active current pane.
    let temp_pane_spec = format!("{}.0", opt.window_name);
    tmux::swap_pane_with(&temp_pane_spec)?;

    let selections = copyrat::run(buffer, &opt.cli_options);

    tmux::swap_pane_with(&temp_pane_spec)?;

    // TODO: consider getting rid of multi-selection mode.

    // Execute a command on each group of selections (normal and uppercased).
    let (normal_selections, uppercased_selections): (Vec<(String, bool)>, Vec<(String, bool)>) =
        selections
            .into_iter()
            .partition(|(_text, uppercased)| !*uppercased);

    let buffer_selections: String = normal_selections
        .into_iter()
        .map(|(text, _)| text)
        .collect::<Vec<_>>()
        .join("\n");

    if buffer_selections.len() > 0 {
        let args = vec!["set-buffer", &buffer_selections];
        // Simply execute the command as is, and let the program crash on
        // potential errors because it is not our responsibility.
        process::execute("tmux", &args).unwrap();
    }

    let buffer_selections: String = uppercased_selections
        .into_iter()
        .map(|(text, _)| text)
        .collect::<Vec<_>>()
        .join("\n");

    if buffer_selections.len() > 0 {
        let args = vec!["set-buffer", &buffer_selections];
        // Simply execute the command as is, and let the program crash on
        // potential errors because it is not our responsibility.
        process::execute("tmux", &args).unwrap();

        let args = vec!["paste-buffer", "-t", active_pane.id.as_str()];
        // Simply execute the command as is, and let the program crash on
        // potential errors because it is not our responsibility.
        process::execute("tmux", &args).unwrap();
    }

    Ok(())
}
