//! This module defines the regex patterns.
//!
//! All patterns must have one capture group. The first group is used.

use crate::{error::Error, Result};

pub(super) const EXCLUDE_PATTERNS: [(&str, &str); 1] =
    [("ansi_colors", r"[[:cntrl:]]\[([0-9]{1,2};)?([0-9]{1,2})?m")];

/// Holds all the regex patterns that are currently supported.
///
/// The email address was obtained at https://www.regular-expressions.info/email.html.
/// Some others were obtained from Ferran Basora, the rest is by me.
pub(super) const PATTERNS: [(&str, &str); 20] = [
    ("markdown-url", r"\[[^]]*\]\(([^)]+)\)"),
    (
        "url",
        r"((https?://|git@|git://|ssh://|ftp://|file:///)[^ \(\)\[\]\{\}]+)",
    ),
    ("email", r"\b([A-z0-9._%+-]+@[A-z0-9.-]+\.[A-z]{2,})\b"),
    ("diff-a", r"--- a/([^ ]+)"),
    ("diff-b", r"\+\+\+ b/([^ ]+)"),
    ("docker", r"sha256:([0-9a-f]{64})"),
    ("path", r"(([.\w\-@~]+)?(/[.\w\-@]+)+)"),
    ("hexcolor", r"(#[0-9a-fA-F]{6})"),
    (
        "uuid",
        r"([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})",
    ),
    (
        "version",
        r"(v?\d{1,4}\.\d{1,4}(\.\d{1,4})?(-(alpha|beta|rc)(\.\d)?)?)[^.0-9s]",
    ),
    ("ipfs", r"(Qm[0-9a-zA-Z]{44})"),
    ("sha", r"([0-9A-f]{7,40})"),
    ("ipv4", r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})"),
    ("ipv6", r"([A-f0-9:]+:+[A-f0-9:]+[%\w\d]+)"),
    ("pointer-address", r"(0x[0-9a-fA-F]+)"),
    (
        "datetime",
        r"(\d{4}-?\d{2}-?\d{2}([ T]\d{2}:\d{2}:\d{2}(\.\d{3,9})?)?)",
    ),
    ("quoted-single", r#"'([^']+)'"#),
    ("quoted-double", r#""([^"]+)""#),
    ("quoted-tick", r#"`([^`]+)`"#),
    ("digits", r"([0-9]{4,})"),
];

/// Type-safe string Pattern Name (newtype).
#[derive(Debug)]
pub struct NamedPattern(pub String, pub String);

/// Parse a name string into `NamedPattern`, used during CLI parsing.
pub(crate) fn parse_pattern_name(src: &str) -> Result<NamedPattern> {
    match PATTERNS.iter().find(|&(name, _pattern)| name == &src) {
        Some((name, pattern)) => Ok(NamedPattern(name.to_string(), pattern.to_string())),
        None => Err(Error::UnknownPatternName),
    }
}
