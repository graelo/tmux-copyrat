use std::fmt;
use std::str::FromStr;

use clap::Args;
use termion::color as tcolor;

use crate::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub struct Color(Option<u8>);

impl tcolor::Color for Color {
    #[inline]
    fn write_fg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(value) => write!(f, "\x1B[38;5;{value}m"),
            None => write!(f, "\x1B[39m"),
        }
    }

    #[inline]
    fn write_bg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(value) => write!(f, "\x1B[48;5;{value}m"),
            None => write!(f, "\x1B[49m"),
        }
    }
}

pub(crate) static BLACK: Color = Color(Some(0));
pub(crate) static RED: Color = Color(Some(1));
pub(crate) static GREEN: Color = Color(Some(2));
pub(crate) static YELLOW: Color = Color(Some(3));
pub(crate) static BLUE: Color = Color(Some(4));
pub(crate) static MAGENTA: Color = Color(Some(5));
pub(crate) static CYAN: Color = Color(Some(6));
pub(crate) static WHITE: Color = Color(Some(7));
pub(crate) static BRIGHTBLACK: Color = Color(Some(8));
pub(crate) static BRIGHTRED: Color = Color(Some(9));
pub(crate) static BRIGHTGREEN: Color = Color(Some(10));
pub(crate) static BRIGHTYELLOW: Color = Color(Some(11));
pub(crate) static BRIGHTBLUE: Color = Color(Some(12));
pub(crate) static BRIGHTMAGENTA: Color = Color(Some(13));
pub(crate) static BRIGHTCYAN: Color = Color(Some(14));
pub(crate) static BRIGHTWHITE: Color = Color(Some(15));
pub(crate) static RESET: Color = Color(None);

impl FromStr for Color {
    type Err = Error;

    fn from_str(src: &str) -> std::result::Result<Self, Self::Err> {
        match src {
            "black" => Ok(BLACK),
            "red" => Ok(RED),
            "green" => Ok(GREEN),
            "yellow" => Ok(YELLOW),
            "blue" => Ok(BLUE),
            "magenta" => Ok(MAGENTA),
            "cyan" => Ok(CYAN),
            "white" => Ok(WHITE),
            "bright-black" | "brightblack" => Ok(BRIGHTBLACK),
            "bright-red" | "brightred" => Ok(BRIGHTRED),
            "bright-green" | "brightgreen" => Ok(BRIGHTGREEN),
            "bright-yellow" | "brightyellow" => Ok(BRIGHTYELLOW),
            "bright-blue" | "brightblue" => Ok(BRIGHTBLUE),
            "bright-magenta" | "brightmagenta" => Ok(BRIGHTMAGENTA),
            "bright-cyan" | "brightcyan" => Ok(BRIGHTCYAN),
            "bright-white" | "brightwhite" => Ok(BRIGHTWHITE),
            "none" => Ok(RESET),
            _ => Err(Error::UnknownColor),
        }
    }
}

pub fn parse_color(src: &str) -> Result<Color> {
    Color::from_str(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_color_fg() {
        let actual = format!("{}{}", tcolor::Fg(Color::from_str("green").unwrap()), "foo");
        let expected = format!("{}{}", tcolor::Fg(tcolor::Green), "foo");

        assert_eq!(actual, expected);
    }

    #[test]
    fn span_color_bg() {
        let actual = format!("{}{}", tcolor::Bg(Color::from_str("green").unwrap()), "foo");
        let expected = format!("{}{}", tcolor::Bg(tcolor::Green), "foo");

        assert_eq!(actual, expected);
    }

    #[test]
    fn no_span_color() {
        assert!(
            Color::from_str("wat").is_err(),
            "this color should not exist"
        );
    }
}

/// Holds color-related data.
///
/// - `focus_*` colors are used to render the currently focused text span.
/// - `normal_*` colors are used to render other text spans.
/// - `hint_*` colors are used to render the hints.
#[derive(Args, Debug)]
// #[clap(about)] // Needed to avoid this doc comment to be used as overall `about`.
pub struct UiColors {
    /// Foreground color for base text.
    #[arg(long, default_value = "bright-cyan", value_parser(parse_color))]
    pub text_fg: Color,

    /// Background color for base text.
    #[clap(long, default_value = "none", value_parser(parse_color))]
    pub text_bg: Color,

    /// Foreground color for spans.
    #[clap(long, default_value = "blue", value_parser(parse_color))]
    pub span_fg: Color,

    /// Background color for spans.
    #[clap(long, default_value = "none", value_parser(parse_color))]
    pub span_bg: Color,

    /// Foreground color for the focused span.
    #[clap(long, default_value = "magenta", value_parser(parse_color))]
    pub focused_fg: Color,

    /// Background color for the focused span.
    #[clap(long, default_value = "none", value_parser(parse_color))]
    pub focused_bg: Color,

    /// Foreground color for hints.
    #[clap(long, default_value = "yellow", value_parser(parse_color))]
    pub hint_fg: Color,

    /// Background color for hints.
    #[clap(long, default_value = "none", value_parser(parse_color))]
    pub hint_bg: Color,
}
