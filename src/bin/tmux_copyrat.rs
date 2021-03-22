use copyrat::{
    config::extended::{ConfigExt, OutputDestination},
    error, tmux,
    ui::Selection,
};

///
fn main() -> Result<(), error::ParseError> {
    let config = ConfigExt::initialize()?;

    // Identify active pane and capture its content.
    let panes: Vec<tmux::Pane> = tmux::list_panes()?;

    let active_pane = panes
        .into_iter()
        .find(|p| p.is_active)
        .expect("Exactly one tmux pane should be active in the current window.");

    let buffer = tmux::capture_pane(&active_pane, &config.capture_region)?;
    let lines = buffer.split('\n').collect::<Vec<_>>();

    // We have to dance a little with Panes, because this process' i/o streams
    // are connected to the pane in the window newly created for us, instead
    // of the active current pane.
    let temp_pane_spec = format!("{}.0", config.window_name);
    tmux::swap_pane_with(&temp_pane_spec)?;

    let selection = copyrat::run(&lines, &config.basic_config);

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
                        .pipe(duct::cmd!(config.clipboard_exe))
                        .read()?;
                }
            }
        }
    }

    Ok(())
}
