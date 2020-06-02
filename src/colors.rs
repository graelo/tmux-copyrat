use crate::error;
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
