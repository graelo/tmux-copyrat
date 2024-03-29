use crate::config::extended::OutputDestination;

/// Represents the text selected by the user, along with if it was uppercased
/// and the output destination (Tmux buffer or Clipboard).
pub struct Selection {
    pub text: String,
    pub uppercased: bool,
    pub output_destination: OutputDestination,
}
