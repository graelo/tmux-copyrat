//! This module provides types and functions to use Tmux.
//!
//! The main use cases are running Tmux commands & parsing Tmux panes
//! information.

use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::config::tmux_bridge::CaptureRegion;
use crate::error::ParseError;

#[derive(Debug, PartialEq)]
pub struct Pane {
    /// Pane identifier, e.g. `%37`.
    pub id: PaneId,
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

impl FromStr for Pane {
    type Err = ParseError;

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
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = src.split(':').collect();
        assert_eq!(items.len(), 5, "tmux should have returned 5 items per line");

        let mut iter = items.iter();

        // Pane id must be start with '%' followed by a `u32`
        let id_str = iter.next().unwrap();
        let id = PaneId::from_str(id_str)?;
        // if !id_str.starts_with('%') {
        //     return Err(ParseError::ExpectedPaneIdMarker);
        // }
        // let id = id_str[1..].parse::<u32>()?;
        // let id = format!("%{}", id);

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

#[derive(Debug, PartialEq)]
pub struct PaneId(String);

impl FromStr for PaneId {
    type Err = ParseError;

    /// Parse into PaneId. The `&str` must be start with '%'
    /// followed by a `u32`.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        if !src.starts_with('%') {
            return Err(ParseError::ExpectedPaneIdMarker);
        }
        let id = src[1..].parse::<u32>()?;
        let id = format!("%{}", id);
        Ok(PaneId(id))
    }
}

impl PaneId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Returns a list of `Pane` from the current tmux session.
pub fn list_panes() -> Result<Vec<Pane>, ParseError> {
    let args = vec![
        "list-panes",
        "-F",
        "#{pane_id}:#{?pane_in_mode,true,false}:#{pane_height}:#{scroll_position}:#{?pane_active,true,false}",
        ];

    let output = duct::cmd("tmux", &args).read()?;

    // Each call to `Pane::parse` returns a `Result<Pane, _>`. All results
    // are collected into a Result<Vec<Pane>, _>, thanks to `collect()`.
    let result: Result<Vec<Pane>, ParseError> = output
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(|line| Pane::from_str(line))
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
    let output = duct::cmd!("tmux", "show", "-g").read()?;
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

/// Returns the entire Pane content as a `String`.
///
/// The provided `region` specifies if the visible area is captured, or the
/// entire history.
///
/// # TODO
///
/// Capture with `capture-pane -J` joins wrapped lines.
///
/// # Note
///
/// If the pane is in normal mode, capturing the visible area can be done
/// without extra arguments (default behavior of `capture-pane`), but if the
/// pane is in copy mode, we need to take into account the current scroll
/// position. To support both cases, the implementation always provides those
/// parameters to tmux.
pub fn capture_pane(pane: &Pane, region: &CaptureRegion) -> Result<String, ParseError> {
    let mut args = format!("capture-pane -t {pane_id} -J -p", pane_id = pane.id);

    let region_str = match region {
        CaptureRegion::VisibleArea => {
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

    let output = duct::cmd("tmux", &args).read()?;
    Ok(output)
}

/// Ask tmux to swap the current Pane with the target_pane (uses Tmux format).
pub fn swap_pane_with(target_pane: &str) -> Result<(), ParseError> {
    // -Z: keep the window zoomed if it was zoomed.
    duct::cmd!("tmux", "swap-pane", "-Z", "-s", target_pane).run()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Pane;
    use super::PaneId;
    use crate::error;
    use std::str::FromStr;

    #[test]
    fn test_parse_pass() {
        let output = vec!["%52:false:62:3:false", "%53:false:23::true"];
        let panes: Result<Vec<Pane>, error::ParseError> =
            output.iter().map(|&line| Pane::from_str(line)).collect();
        let panes = panes.expect("Could not parse tmux panes");

        let expected = vec![
            Pane {
                id: PaneId::from_str("%52").unwrap(),
                in_mode: false,
                height: 62,
                scroll_position: 3,
                is_active: false,
            },
            Pane {
                // id: PaneId::from_str("%53").unwrap(),
                id: PaneId(String::from("%53")),
                in_mode: false,
                height: 23,
                scroll_position: 0,
                is_active: true,
            },
        ];

        assert_eq!(panes, expected);
    }
}
