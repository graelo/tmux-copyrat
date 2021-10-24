use clap::{ArgEnum, Parser};

/// Describes if, during rendering, a hint should aligned to the leading edge of
/// the matched text, or to its trailing edge.
#[derive(Debug, Clone, ArgEnum, Parser)]
pub enum HintAlignment {
    Leading,
    Trailing,
}
