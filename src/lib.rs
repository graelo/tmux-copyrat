//! A tmux plugin for copy-pasting spans of text from a tmux pane's history
//! into a clipboard.
//!
//! This crate is consumed as two binaries rather than as a library API:
//!
//! - `tmux-copyrat`: captures the active tmux pane, displays keyboard hints
//!   over every matching span, and copies the selected one to the tmux buffer
//!   or the system clipboard.
//! - `copyrat`: the same span picker without any tmux dependency, reading
//!   from stdin and writing the selection to stdout.
//!
//! End-user documentation — installation, configuration, key bindings, and
//! the list of matched patterns — lives in the repository:
//!
//! - [README](https://github.com/graelo/tmux-copyrat#readme)
//! - [Installation](https://github.com/graelo/tmux-copyrat/blob/main/INSTALLATION.md)
//! - [Configuration](https://github.com/graelo/tmux-copyrat/blob/main/CONFIGURATION.md)

pub mod config;
pub mod error;
pub mod textbuf;
pub mod tmux;
pub mod ui;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Run copyrat on an input string `buffer`, configured by `Opt`.
///
/// # Note
///
/// Maybe the decision to take ownership of the buffer is a bit bold.
pub fn run(lines: &[&str], opt: &config::basic::Config) -> Option<ui::Selection> {
    let model = textbuf::Model::new(
        lines,
        &opt.alphabet,
        opt.use_all_patterns,
        &opt.named_patterns,
        &opt.custom_patterns,
        opt.reverse,
        opt.unique_hint,
    );

    if model.spans.is_empty() {
        return None;
    }

    let selection: Option<ui::Selection> = {
        let mut ui = ui::ViewController::new(
            &model,
            opt.focus_wrap_around,
            &opt.default_output,
            &opt.colors,
            &opt.hint_alignment,
            opt.hint_style(),
            ui::MultiSelectConfig {
                enabled: opt.multi_select,
                separator: opt.separator.clone(),
            },
        );

        ui.present()
    };

    selection
}
