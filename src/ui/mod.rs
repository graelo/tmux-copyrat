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

mod vc;
pub use vc::ViewController;
pub use vc::{HintAlignment, HintStyle};
