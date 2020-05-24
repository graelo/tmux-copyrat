use std::fmt;

#[derive(Debug)]
pub enum ParseError {
  ExpectedSurroundingPair,
  UnknownAlphabet,
  UnknownColor,
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ParseError::ExpectedSurroundingPair => write!(f, "Expected 2 chars"),
      ParseError::UnknownAlphabet => write!(f, "Expected a known alphabet"),
      ParseError::UnknownColor => write!(f, "Expected ANSI color name (magenta, cyan, black, ...)"),
    }
  }
}
