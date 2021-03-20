use clap::Clap;
use std::str::FromStr;

use crate::error::ParseError;

/// Describes if, during rendering, a hint should aligned to the leading edge of
/// the matched text, or to its trailing edge.
#[derive(Debug, Clap)]
pub enum HintAlignment {
    Leading,
    Trailing,
}

impl FromStr for HintAlignment {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<HintAlignment, ParseError> {
        match s {
            "leading" => Ok(HintAlignment::Leading),
            "trailing" => Ok(HintAlignment::Trailing),
            _ => Err(ParseError::ExpectedString(String::from(
                "leading or trailing",
            ))),
        }
    }
}
