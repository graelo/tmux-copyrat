// use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Expected 2 chars")]
    ExpectedSurroundingPair,

    #[error("Unknown alphabet")]
    UnknownAlphabet,

    #[error("Unknown ANSI color name: allowed values are magenta, cyan, black, ...")]
    UnknownColor,

    #[error("Unknown pattern name")]
    UnknownPatternName,

    #[error("Invalid custom regex: {0}")]
    InvalidCustomRegex(#[source] regex::Error),

    #[error("Custom regex must contain a capture group")]
    CustomRegexMissingCaptureGroup,

    #[error("Expected a pane id marker")]
    ExpectedPaneIdMarker,

    #[error("Expected five colon-separated tmux pane fields")]
    ExpectedPaneFieldCount,

    #[error("Failed parsing integer")]
    ExpectedInt {
        #[from]
        source: std::num::ParseIntError,
    },

    #[error("Failed to parse bool")]
    ExpectedBool {
        #[from]
        source: std::str::ParseBoolError,
    },

    #[error("Expected the string `{0}`")]
    ExpectedString(String),

    #[error("Expected the value to be within `{0}`")]
    ExpectedEnumVariant(String),

    #[error("IOError: `{source}`")]
    Io {
        #[from]
        source: std::io::Error,
    },
}
