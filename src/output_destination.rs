use std::fmt;

/// Describes the type of buffer the selected should be copied to: either a
/// tmux buffer or the system clipboard.
#[derive(Clone)]
pub enum OutputDestination {
    /// The selection will be copied to the tmux buffer.
    Tmux,
    /// The selection will be copied to the system clipboard.
    Clipboard,
}

impl OutputDestination {
    /// Toggle between the variants of `OutputDestination`.
    pub fn toggle(&mut self) {
        match *self {
            Self::Tmux => *self = Self::Clipboard,
            Self::Clipboard => *self = Self::Tmux,
        }
    }
}

impl fmt::Display for OutputDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tmux => write!(f, "tmux"),
            Self::Clipboard => write!(f, "clipboard"),
        }
    }
}
