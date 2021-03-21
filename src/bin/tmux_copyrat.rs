use clap::Clap;
use std::collections::HashMap;
use std::str::FromStr;

use copyrat::{
    comm::{tmux, OutputDestination},
    config::CliOpt,
    error,
    ui::Selection,
};

/// Main configuration, parsed from command line.
#[derive(Clap, Debug)]
#[clap(author, about, version)]
struct BridgeOpt {
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

    /// Name of the copy-to-clipboard executable.
    ///
    /// If during execution, the output destination is set to be clipboard,
    /// then copyrat will pipe the selected text to this executable.
    #[clap(long, default_value = "pbcopy")]
    clipboard_exe: String,

    // Include CLI Options
    #[clap(flatten)]
    cli_options: CliOpt,
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

///
fn main() -> Result<(), error::ParseError> {
    let mut opt = BridgeOpt::parse();

    if !opt.ignore_options_from_tmux {
        let tmux_options: HashMap<String, String> = tmux::get_options("@copyrat-")?;

        // Override default values with those coming from tmux.
        opt.merge_map(&tmux_options)?;
    }

    // Identify active pane and capture its content.
    let panes: Vec<tmux::Pane> = tmux::list_panes()?;

    let active_pane = panes
        .into_iter()
        .find(|p| p.is_active)
        .expect("Exactly one tmux pane should be active in the current window.");

    let buffer = tmux::capture_pane(&active_pane, &opt.capture_region)?;

    // We have to dance a little with Panes, because this process' i/o streams
    // are connected to the pane in the window newly created for us, instead
    // of the active current pane.
    let temp_pane_spec = format!("{}.0", opt.window_name);
    tmux::swap_pane_with(&temp_pane_spec)?;

    let selection = copyrat::run(buffer, &opt.cli_options);

    tmux::swap_pane_with(&temp_pane_spec)?;

    // Finally copy selection to the output destination (tmux buffer or
    // clipboard), and paste it to the active buffer if it was uppercased.

    match selection {
        None => return Ok(()),
        Some(Selection {
            text,
            uppercased,
            output_destination,
        }) => {
            if uppercased {
                duct::cmd!("tmux", "send-keys", "-t", active_pane.id.as_str(), &text).run()?;
            }

            match output_destination {
                OutputDestination::Tmux => {
                    duct::cmd!("tmux", "set-buffer", &text).run()?;
                }
                OutputDestination::Clipboard => {
                    duct::cmd!("echo", "-n", &text)
                        .pipe(duct::cmd!(opt.clipboard_exe))
                        .read()?;
                }
            }
        }
    }

    Ok(())
}
