use crate::error;

pub const EXCLUDE_PATTERNS: [(&'static str, &'static str); 1] =
    [("ansi_colors", r"[[:cntrl:]]\[([0-9]{1,2};)?([0-9]{1,2})?m")];

pub const PATTERNS: [(&'static str, &'static str); 14] = [
    ("markdown_url", r"\[[^]]*\]\(([^)]+)\)"),
    (
        "url",
        r"((https?://|git@|git://|ssh://|ftp://|file:///)[^ \(\)\[\]\{\}]+)",
    ),
    ("diff_a", r"--- a/([^ ]+)"),
    ("diff_b", r"\+\+\+ b/([^ ]+)"),
    ("docker", r"sha256:([0-9a-f]{64})"),
    ("path", r"(([.\w\-@~]+)?(/[.\w\-@]+)+)"),
    ("hexcolor", r"#[0-9a-fA-F]{6}"),
    (
        "uuid",
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
    ),
    ("ipfs", r"Qm[0-9a-zA-Z]{44}"),
    ("sha", r"[0-9a-f]{7,40}"),
    ("ip", r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"),
    ("ipv6", r"[A-f0-9:]+:+[A-f0-9:]+[%\w\d]+"),
    ("address", r"0x[0-9a-fA-F]+"),
    ("number", r"[0-9]{4,}"),
];

/// Type-safe string Pattern Name (newtype).
#[derive(Debug)]
pub struct NamedPattern(pub String, pub String);

/// Parse a name string into `NamedPattern`, used during CLI parsing.
pub fn parse_pattern_name(src: &str) -> Result<NamedPattern, error::ParseError> {
    match PATTERNS.iter().find(|&(name, _pattern)| name == &src) {
        Some((name, pattern)) => Ok(NamedPattern(name.to_string(), pattern.to_string())),
        None => Err(error::ParseError::UnknownPatternName),
    }
}
