use clap::Clap;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

use copyrat::error::ParseError;
use copyrat::process;

#[derive(Debug, PartialEq)]
pub struct Pane {
    /// Pane identifier.
    pub id: u32,
    /// Describes if the pane is in some mode.
    pub in_mode: bool,
    /// Number of lines in the pane.
    pub height: u32,
    /// Optional offset from the bottom if the pane is in some mode.
    ///
    /// When a pane is in copy mode, scrolling up changes the
    /// `scroll_position`. If the pane is in normal mode, or unscrolled,
    /// then `0` is returned.
    pub scroll_position: u32,
    /// Describes if the pane is currently active (focused).
    pub is_active: bool,
}

impl Pane {
    /// Parse a string containing tmux panes status into a new `Pane`.
    ///
    /// This returns a `Result<Pane, ParseError>` as this call can obviously
    /// fail if provided an invalid format.
    ///
    /// The expected format of the tmux status is "%52:false:62:3:false",
    /// or "%53:false:23::true".
    ///
    /// This status line is obtained with `tmux list-panes -F '#{pane_id}:#{?pane_in_mode,true,false}:#{pane_height}:#{scroll_position}:#{?pane_active,true,false}'`.
    ///
    /// For definitions, look at `Pane` type,
    /// and at the tmux man page for definitions.
    pub fn parse(src: &str) -> Result<Pane, ParseError> {
        let items: Vec<&str> = src.split(':').collect();
        assert_eq!(items.len(), 5, "tmux should have returned 5 items per line");

        let mut iter = items.iter();

        let id_str = iter.next().unwrap();
        if !id_str.starts_with('%') {
            return Err(ParseError::ExpectedPaneIdMarker);
        }
        let id = id_str[1..].parse::<u32>()?;

        let in_mode = iter.next().unwrap().parse::<bool>()?;

        let height = iter.next().unwrap().parse::<u32>()?;

        let scroll_position = iter.next().unwrap();
        let scroll_position = if scroll_position.is_empty() {
            "0"
        } else {
            scroll_position
        };
        let scroll_position = scroll_position.parse::<u32>()?;

        let is_active = iter.next().unwrap().parse::<bool>()?;

        Ok(Pane {
            id,
            in_mode,
            height,
            scroll_position,
            is_active,
        })
    }
}

/// Returns a list of `Pane` from the current tmux session.
pub fn list_panes() -> Result<Vec<Pane>, ParseError> {
    let args = vec![
        "list-panes",
        "-F",
        "#{pane_id}:#{?pane_in_mode,true,false}:#{pane_height}:#{scroll_position}:#{?pane_active,true,false}",
        ];

    let output = process::execute("tmux", &args)?;

    // Each call to `Pane::parse` returns a `Result<Pane, _>`. All results
    // are collected into a Result<Vec<Pane>, _>, thanks to `collect()`.
    let result: Result<Vec<Pane>, ParseError> = output
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(|line| Pane::parse(line))
        .collect();

    result
}

/// Returns tmux global options as a `HashMap`. The prefix argument is for
/// convenience, in order to target only some of our options. For instance,
/// `get_options("@copyrat-")` will return a `HashMap` which keys are tmux options names like `@copyrat-command`, and associated values.
///
/// # Example
/// ```get_options("@copyrat-")```
pub fn get_options(prefix: &str) -> Result<HashMap<String, String>, ParseError> {
    let args = vec!["show", "-g"];

    let output = process::execute("tmux", &args)?;
    let lines: Vec<&str> = output.split('\n').collect();

    let pattern = format!(r#"{prefix}([\w\-0-9]+) "?(\w+)"?"#, prefix = prefix);
    let re = Regex::new(&pattern).unwrap();

    let args: HashMap<String, String> = lines
        .iter()
        .flat_map(|line| match re.captures(line) {
            None => None,
            Some(captures) => {
                let key = captures[1].to_string();
                let value = captures[2].to_string();
                Some((key, value))
            }
        })
        .collect();

    Ok(args)
}

// pub fn toto() {
//         let options_command = vec!["tmux", "show", "-g"];
//         let params: Vec<String> = options_command.iter().map(|arg| arg.to_string()).collect();
//         let options = self.executor.execute(params);
//         let lines: Vec<&str> = options.split('\n').collect();

//         let pattern = Regex::new(r#"@thumbs-([\w\-0-9]+) "?(\w+)"?"#).unwrap();

//         let args = lines
//             .iter()
//             .flat_map(|line| {
//                 if let Some(captures) = pattern.captures(line) {
//                     let name = captures.get(1).unwrap().as_str();
//                     let value = captures.get(2).unwrap().as_str();

//                     let boolean_params = vec!["reverse", "unique", "contrast"];

//                     if boolean_params.iter().any(|&x| x == name) {
//                         return vec![format!("--{}", name)];
//                     }

//                     let string_params = vec![
//                         "position",
//                         "fg-color",
//                         "bg-color",
//                         "hint-bg-color",
//                         "hint-fg-color",
//                         "select-fg-color",
//                         "select-bg-color",
//                     ];

//                     if string_params.iter().any(|&x| x == name) {
//                         return vec![format!("--{}", name), format!("'{}'", value)];
//                     }

//                     if name.starts_with("regexp") {
//                         return vec!["--regexp".to_string(), format!("'{}'", value)];
//                     }
// }
// };}

#[derive(Clap, Debug)]
pub enum CaptureRegion {
    /// The entire history.
    ///
    /// This will end up sending `-S - -E -` to `tmux capture-pane`.
    EntireHistory,
    /// The visible area.
    VisibleArea,
    ///// Region from start line to end line
    /////
    ///// This works as defined in tmux's docs (order does not matter).
    //Region(i32, i32),
}

impl FromStr for CaptureRegion {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "leading" => Ok(CaptureRegion::EntireHistory),
            "trailing" => Ok(CaptureRegion::VisibleArea),
            _ => Err(ParseError::ExpectedString(String::from(
                "entire-history or visible-area",
            ))),
        }
    }
}

/// Returns the entire Pane content as a `String`.
///
/// `CaptureRegion` specifies if the visible area is captured, or the entire
/// history.
///
/// # Note
///
/// If the pane is in normal mode, capturing the visible area can be done
/// without extra arguments (default behavior of `capture-pane`), but if the
/// pane is in copy mode, we need to take into account the current scroll
/// position. To support both cases, the implementation always provides those
/// parameters to tmux.
pub fn capture_pane(pane: &Pane, region: &CaptureRegion) -> Result<String, ParseError> {
    let mut args = format!("capture-pane -t %{id} -p", id = pane.id);

    let region_str = match region {
        CaptureRegion::VisibleArea => {
            // Will capture the visible area.
            // Providing start/end helps support both copy and normal modes.
            format!(
                " -S {start} -E {end}",
                start = pane.scroll_position,
                end = pane.height - pane.scroll_position - 1
            )
        }
        CaptureRegion::EntireHistory => String::from(" -S - -E -"),
    };

    args.push_str(&region_str);

    let args: Vec<&str> = args.split(' ').collect();

    let output = process::execute("tmux", &args)?;
    Ok(output)

    // format!(
    // "tmux capture-pane -t {} -p{} | {}/target/release/thumbs -f '%U:%H' -t {} {}; tmux swap-pane -t {}; tmux wait-for -S {}",
    // active_pane_id,
    // scroll_params,
}

/// Creates a new named window in the background (without switching to it) and
/// returns a `Pane` describing the newly created pane.
///
/// # Note
///
/// Returning a new `Pane` seems overkill, given we mostly take care of its
/// Id, but it is cleaner.
pub fn create_new_window(name: &str) -> Result<Pane, ParseError> {
    let args = vec!["new-window", "-P", "-d", "-n", name, "-F",
        "#{pane_id}:#{?pane_in_mode,true,false}:#{pane_height}:#{scroll_position}:#{?pane_active,true,false}"];

    let output = process::execute("tmux", &args)?;

    let pane = Pane::parse(output.trim_end())?; // trim last '\n' as it would create an empty line

    Ok(pane)
}

/// Ask tmux to swap two `Pane`s and change the active pane to be the target
/// `Pane`.
pub fn swap_panes(pane_a: &Pane, pane_b: &Pane) -> Result<(), ParseError> {
    let pa_id = format!("%{}", pane_a.id);
    let pb_id = format!("%{}", pane_b.id);

    let args = vec!["swap-pane", "-s", &pa_id, "-t", &pb_id];

    process::execute("tmux", &args)?;

    Ok(())
}

/// Ask tmux to kill the provided `Pane`.
pub fn kill_pane(pane: &Pane) -> Result<(), ParseError> {
    let p_id = format!("%{}", pane.id);

    let args = vec!["kill-pane", "-t", &p_id];

    process::execute("tmux", &args)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Pane;
    use copyrat::error;

    #[test]
    fn test_parse_pass() {
        let output = vec!["%52:false:62:3:false", "%53:false:23::true"];
        let panes: Result<Vec<Pane>, error::ParseError> =
            output.iter().map(|&line| Pane::parse(line)).collect();
        let panes = panes.expect("Could not parse tmux panes");

        let expected = vec![
            Pane {
                id: 52,
                in_mode: false,
                height: 62,
                scroll_position: 3,
                is_active: false,
            },
            Pane {
                id: 53,
                in_mode: false,
                height: 23,
                scroll_position: 0,
                is_active: true,
            },
        ];

        assert_eq!(panes, expected);
    }
}
