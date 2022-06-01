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

    let default_output_destination = config::extended::OutputDestination::Tmux;

    let selection: Option<ui::Selection> = {
        let mut ui = ui::ViewController::new(
            &model,
            opt.focus_wrap_around,
            default_output_destination,
            &opt.colors,
            &opt.hint_alignment,
            opt.hint_style(),
        );

        ui.present()
    };

    selection
}
