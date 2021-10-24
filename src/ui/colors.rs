use clap::Parser;
use termion::color;

use crate::{error::ParseError, Result};

pub fn parse_color(src: &str) -> Result<Box<dyn color::Color>> {
    match src {
        "black" => Ok(Box::new(color::Black)),
        "red" => Ok(Box::new(color::Red)),
        "green" => Ok(Box::new(color::Green)),
        "yellow" => Ok(Box::new(color::Yellow)),
        "blue" => Ok(Box::new(color::Blue)),
        "magenta" => Ok(Box::new(color::Magenta)),
        "cyan" => Ok(Box::new(color::Cyan)),
        "white" => Ok(Box::new(color::White)),
        "bright-black" | "brightblack" => Ok(Box::new(color::LightBlack)),
        "bright-red" | "brightred" => Ok(Box::new(color::LightRed)),
        "bright-green" | "brightgreen" => Ok(Box::new(color::LightGreen)),
        "bright-yellow" | "brightyellow" => Ok(Box::new(color::LightYellow)),
        "bright-blue" | "brightblue" => Ok(Box::new(color::LightBlue)),
        "bright-magenta" | "brightmagenta" => Ok(Box::new(color::LightMagenta)),
        "bright-cyan" | "brightcyan" => Ok(Box::new(color::LightCyan)),
        "bright-white" | "brightwhite" => Ok(Box::new(color::LightWhite)),
        "none" => Ok(Box::new(color::Reset)),
        _ => Err(ParseError::UnknownColor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_color() {
        let text1 = format!(
            "{}{}",
            color::Fg(parse_color("green").unwrap().as_ref()),
            "foo"
        );
        let text2 = format!("{}{}", color::Fg(color::Green), "foo");

        assert_eq!(text1, text2);
    }

    #[test]
    fn no_span_color() {
        assert!(parse_color("wat").is_err(), "this color should not exist");
    }
}

/// Holds color-related data.
///
/// - `focus_*` colors are used to render the currently focused text span.
/// - `normal_*` colors are used to render other text spans.
/// - `hint_*` colors are used to render the hints.
#[derive(Parser, Debug)]
#[clap(about)] // Needed to avoid this doc comment to be used as overall `about`.
pub struct UiColors {
    /// Foreground color for base text.
    #[clap(long, default_value = "bright-cyan", parse(try_from_str = parse_color))]
    pub text_fg: Box<dyn color::Color>,

    /// Background color for base text.
    #[clap(long, default_value = "none", parse(try_from_str = parse_color))]
    pub text_bg: Box<dyn color::Color>,

    /// Foreground color for spans.
    #[clap(long, default_value = "blue",
                parse(try_from_str = parse_color))]
    pub span_fg: Box<dyn color::Color>,

    /// Background color for spans.
    #[clap(long, default_value = "none",
                parse(try_from_str = parse_color))]
    pub span_bg: Box<dyn color::Color>,

    /// Foreground color for the focused span.
    #[clap(long, default_value = "magenta",
                parse(try_from_str = parse_color))]
    pub focused_fg: Box<dyn color::Color>,

    /// Background color for the focused span.
    #[clap(long, default_value = "none",
                parse(try_from_str = parse_color))]
    pub focused_bg: Box<dyn color::Color>,

    /// Foreground color for hints.
    #[clap(long, default_value = "yellow",
                parse(try_from_str = parse_color))]
    pub hint_fg: Box<dyn color::Color>,

    /// Background color for hints.
    #[clap(long, default_value = "none",
                parse(try_from_str = parse_color))]
    pub hint_bg: Box<dyn color::Color>,
}
