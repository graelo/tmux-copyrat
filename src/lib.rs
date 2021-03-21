pub mod comm;
pub mod config;
pub mod error;
pub mod textbuf;
pub mod ui;

/// Run copyrat on an input string `buffer`, configured by `Opt`.
///
/// # Note
///
/// Maybe the decision to take ownership of the buffer is a bit bold.
pub fn run(buffer: String, opt: &config::basic::Config) -> Option<ui::Selection> {
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
            config::basic::HintStyleArg::Bold => Some(ui::HintStyle::Bold),
            config::basic::HintStyleArg::Italic => Some(ui::HintStyle::Italic),
            config::basic::HintStyleArg::Underline => Some(ui::HintStyle::Underline),
            config::basic::HintStyleArg::Surround => {
                let (open, close) = opt.hint_surroundings;
                Some(ui::HintStyle::Surround(open, close))
            }
        },
    };

    let default_output_destination = comm::OutputDestination::Tmux;

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
