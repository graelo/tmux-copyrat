//! The `ui` module is responsible for presenting information to the user and
//! handling keypresses.
//!
//! In particular, the `Ui` struct
//!
//! - renders text, matched text and hints from the structured buffer content
//!   to the screen,
//! - listens for keypress events,
//! - and returns the user selection in the form of a `Selection` struct.
//!
//! Via keypresses the user can
//!
//! - navigate the buffer (in case it is larger than the number of lines in
//!   the terminal)
//! - move the focus from one match to another
//! - select one of the matches
//! - toggle the output destination (tmux buffer or clipboard)
//!

pub mod colors;
pub mod hint_style;
mod selection;
mod vc;

pub use hint_style::HintStyle;
pub use selection::Selection;
pub use vc::HintAlignment;
pub use vc::ViewController;
