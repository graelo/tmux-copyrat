pub const EXCLUDE_PATTERNS: [(&'static str, &'static str); 1] =
    [("ansi_colors", r"[[:cntrl:]]\[([0-9]{1,2};)?([0-9]{1,2})?m")];

pub const PATTERNS: [(&'static str, &'static str); 14] = [
    ("markdown_url", r"\[[^]]*\]\(([^)]+)\)"),
    (
        "url",
        r"((https?://|git@|git://|ssh://|ftp://|file:///)[^ ]+)",
    ),
    ("diff_a", r"--- a/([^ ]+)"),
    ("diff_b", r"\+\+\+ b/([^ ]+)"),
    ("docker", r"sha256:([0-9a-f]{64})"),
    ("path", r"(([.\w\-@~]+)?(/[.\w\-@]+)+)"),
    ("color", r"#[0-9a-fA-F]{6}"),
    (
        "uid",
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
    ),
    ("ipfs", r"Qm[0-9a-zA-Z]{44}"),
    ("sha", r"[0-9a-f]{7,40}"),
    ("ip", r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"),
    ("ipv6", r"[A-f0-9:]+:+[A-f0-9:]+[%\w\d]+"),
    ("address", r"0x[0-9a-fA-F]+"),
    ("number", r"[0-9]{4,}"),
];
