use crate::error;
use clap::Clap;
use termion::color;

pub fn parse_color(src: &str) -> Result<Box<dyn color::Color>, error::ParseError> {
    match src {
        "black" => Ok(Box::new(color::Black)),
        "red" => Ok(Box::new(color::Red)),
        "green" => Ok(Box::new(color::Green)),
        "yellow" => Ok(Box::new(color::Yellow)),
        "blue" => Ok(Box::new(color::Blue)),
        "magenta" => Ok(Box::new(color::Magenta)),
        "cyan" => Ok(Box::new(color::Cyan)),
        "white" => Ok(Box::new(color::White)),
        "bright-black" => Ok(Box::new(color::LightBlack)),
        "bright-red" => Ok(Box::new(color::LightRed)),
        "bright-green" => Ok(Box::new(color::LightGreen)),
        "bright-yellow" => Ok(Box::new(color::LightYellow)),
        "bright-blue" => Ok(Box::new(color::LightBlue)),
        "bright-magenta" => Ok(Box::new(color::LightMagenta)),
        "bright-cyan" => Ok(Box::new(color::LightCyan)),
        "bright-white" => Ok(Box::new(color::LightWhite)),
        // "default" => Ok(Box::new(color::Reset)),
        _ => Err(error::ParseError::UnknownColor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_color() {
        let text1 = format!(
            "{}{}",
            color::Fg(parse_color("green").unwrap().as_ref()),
            "foo"
        );
        let text2 = format!("{}{}", color::Fg(color::Green), "foo");

        assert_eq!(text1, text2);
    }

    #[test]
    fn no_match_color() {
        assert!(parse_color("wat").is_err(), "this color should not exist");
    }
}

/// Holds color-related data.
///
/// - `focus_*` colors are used to render the currently focused matched text.
/// - `normal_*` colors are used to render other matched text.
/// - `hint_*` colors are used to render the hints.
#[derive(Clap, Debug)]
#[clap(about)] // Needed to avoid this doc comment to be used as overall `about`.
pub struct UiColors {
    /// Foreground color for base text.
    #[clap(long, default_value = "bright-cyan", parse(try_from_str = parse_color))]
    pub text_fg: Box<dyn color::Color>,

    /// Background color for base text.
    #[clap(long, default_value = "bright-white", parse(try_from_str = parse_color))]
    pub text_bg: Box<dyn color::Color>,

    /// Foreground color for matches.
    #[clap(long, default_value = "yellow",
                parse(try_from_str = parse_color))]
    pub match_fg: Box<dyn color::Color>,

    /// Background color for matches.
    #[clap(long, default_value = "bright-white",
                parse(try_from_str = parse_color))]
    pub match_bg: Box<dyn color::Color>,

    /// Foreground color for the focused match.
    #[clap(long, default_value = "magenta",
                parse(try_from_str = parse_color))]
    pub focused_fg: Box<dyn color::Color>,

    /// Background color for the focused match.
    #[clap(long, default_value = "bright-white",
                parse(try_from_str = parse_color))]
    pub focused_bg: Box<dyn color::Color>,

    /// Foreground color for hints.
    #[clap(long, default_value = "white",
                parse(try_from_str = parse_color))]
    pub hint_fg: Box<dyn color::Color>,

    /// Background color for hints.
    #[clap(long, default_value = "magenta",
                parse(try_from_str = parse_color))]
    pub hint_bg: Box<dyn color::Color>,
}
