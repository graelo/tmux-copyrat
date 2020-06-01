use std::collections;
use std::fmt;

use regex::Regex;
use sequence_trie::SequenceTrie;

use crate::alphabets::Alphabet;
use crate::regexes::{NamedPattern, EXCLUDE_PATTERNS, PATTERNS};

/// Holds data for the `Ui`.
pub struct Model<'a> {
    pub lines: &'a Vec<&'a str>,
    alphabet: &'a Alphabet,
    named_patterns: &'a Vec<NamedPattern>,
    custom_regexes: &'a Vec<String>,
    pub reverse: bool,
}

impl<'a> Model<'a> {
    pub fn new(
        lines: &'a Vec<&'a str>,
        alphabet: &'a Alphabet,
        named_patterns: &'a Vec<NamedPattern>,
        custom_regexes: &'a Vec<String>,
        reverse: bool,
    ) -> Model<'a> {
        Model {
            lines,
            alphabet,
            named_patterns,
            custom_regexes,
            reverse,
        }
    }

    /// Returns a vector of `Match`es, each corresponding to a pattern match
    /// in the lines, its location (x, y), and associated hint.
    pub fn matches(&self, unique: bool) -> Vec<Match<'a>> {
        let mut raw_matches = self.raw_matches();

        if self.reverse {
            raw_matches.reverse();
        }

        let mut matches = self.associate_hints(&raw_matches, unique);

        if self.reverse {
            matches.reverse();
        }

        matches
    }

    /// Internal function that searches the model's lines for pattern matches.
    /// Returns a vector of `RawMatch`es (text, location, pattern id) without
    /// an associated hint. The hint is attached to `Match`, not to `RawMatch`.
    ///
    /// # Notes
    ///
    /// Custom regexes have priority over other regexes.
    ///
    /// If no named patterns were specified, it will search for all available
    /// patterns from the `PATTERNS` catalog.
    fn raw_matches(&self) -> Vec<RawMatch<'a>> {
        let mut matches = Vec::new();

        let exclude_regexes = EXCLUDE_PATTERNS
            .iter()
            .map(|&(name, pattern)| (name, Regex::new(pattern).unwrap()))
            .collect::<Vec<_>>();

        let custom_regexes = self
            .custom_regexes
            .iter()
            .map(|pattern| {
                (
                    "custom",
                    Regex::new(pattern).expect("Invalid custom regexp"),
                )
            })
            .collect::<Vec<_>>();

        let regexes = if self.named_patterns.is_empty() {
            PATTERNS
                .iter()
                .map(|&(name, pattern)| (name, Regex::new(pattern).unwrap()))
                .collect::<Vec<(&str, regex::Regex)>>()
        } else {
            self.named_patterns
                .iter()
                .map(|NamedPattern(name, pattern)| (name.as_str(), Regex::new(pattern).unwrap()))
                .collect::<Vec<(&str, regex::Regex)>>()
        };

        let all_regexes = [exclude_regexes, custom_regexes, regexes].concat();

        for (index, line) in self.lines.iter().enumerate() {
            // Remainder of the line to be searched for matches.
            // This advances iteratively, until no matches can be found.
            let mut chunk: &str = line;
            let mut offset: i32 = 0;

            // Use all avail regexes to match the chunk and select the match
            // occuring the earliest on the chunk. Save its matched text and
            // position in a `Match` struct.
            loop {
                let chunk_matches = all_regexes
                    .iter()
                    .filter_map(|(&ref name, regex)| match regex.find_iter(chunk).nth(0) {
                        Some(m) => Some((name, regex, m)),
                        None => None,
                    })
                    .collect::<Vec<_>>();

                if chunk_matches.is_empty() {
                    break;
                }

                // First match on the chunk.
                let (name, pattern, matching) = chunk_matches
                    .iter()
                    .min_by(|x, y| x.2.start().cmp(&y.2.start()))
                    .unwrap();

                let text = matching.as_str();

                let captures = pattern
                    .captures(text)
                    .expect("At this stage the regex must have matched.");

                // Handle both capturing and non-capturing patterns.
                let (subtext, substart) = if let Some(capture) = captures.get(1) {
                    (capture.as_str(), capture.start())
                } else {
                    (text, 0)
                };

                // Never hint or break ansi color sequences.
                if *name != "ansi_colors" {
                    matches.push(RawMatch {
                        x: offset + matching.start() as i32 + substart as i32,
                        y: index as i32,
                        pattern: name,
                        text: subtext,
                    });
                }

                chunk = chunk.get(matching.end()..).expect("Unknown chunk");
                offset += matching.end() as i32;
            }
        }

        matches
    }

    /// Associate a hint to each `RawMatch`, returning a vector of `Match`es.
    ///
    /// If `unique` is `true`, all duplicate matches will have the same hint.
    /// For copying matched text, this seems easier and more natural.
    /// If `unique` is `false`, duplicate matches will have their own hint.
    fn associate_hints(&self, raw_matches: &Vec<RawMatch<'a>>, unique: bool) -> Vec<Match<'a>> {
        let hints = self.alphabet.make_hints(raw_matches.len());
        let mut hints_iter = hints.iter();

        let mut result: Vec<Match<'a>> = vec![];

        if unique {
            // Map (text, hint)
            let mut known: collections::HashMap<&str, &str> = collections::HashMap::new();

            for raw_mat in raw_matches {
                let hint: &str = known.entry(raw_mat.text).or_insert(
                    hints_iter
                        .next()
                        .expect("We should have as many hints as necessary, even invisible ones."),
                );

                result.push(Match {
                    x: raw_mat.x,
                    y: raw_mat.y,
                    pattern: raw_mat.pattern,
                    text: raw_mat.text,
                    hint: hint.to_string(),
                });
            }
        } else {
            for raw_mat in raw_matches {
                let hint = hints_iter
                    .next()
                    .expect("We should have as many hints as necessary, even invisible ones.");

                result.push(Match {
                    x: raw_mat.x,
                    y: raw_mat.y,
                    pattern: raw_mat.pattern,
                    text: raw_mat.text,
                    hint: hint.to_string(),
                });
            }
        }

        result
    }

    /// Builds a `SequenceTrie` that helps determine if a sequence of keys
    /// entered by the user corresponds to a match. This kind of lookup
    /// directly returns a reference to the corresponding `Match` if any.
    pub fn build_lookup_trie(matches: &'a Vec<Match<'a>>) -> SequenceTrie<char, usize> {
        let mut trie = SequenceTrie::new();

        for (index, mat) in matches.iter().enumerate() {
            let hint_chars = mat.hint.chars().collect::<Vec<char>>();

            // no need to insert twice the same hint
            if trie.get(&hint_chars).is_none() {
                trie.insert_owned(hint_chars, index);
            }
        }

        trie
    }
}

/// Represents matched text, its location on screen, the pattern that created
/// it, and the associated hint.
pub struct Match<'a> {
    pub x: i32,
    pub y: i32,
    pub pattern: &'a str,
    pub text: &'a str,
    pub hint: String,
}

impl<'a> fmt::Debug for Match<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Match {{ x: {}, y: {}, pattern: {}, text: {}, hint: <{}> }}",
            self.x, self.y, self.pattern, self.text, self.hint,
        )
    }
}

/// Internal surrogate for `Match`, before a Hint has been associated.
struct RawMatch<'a> {
    pub x: i32,
    pub y: i32,
    pub pattern: &'a str,
    pub text: &'a str,
}

impl<'a> fmt::Debug for RawMatch<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RawMatch {{ x: {}, y: {}, pattern: {}, text: {} }}",
            self.x, self.y, self.pattern, self.text,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabets::Alphabet;

    fn split(output: &str) -> Vec<&str> {
        output.split("\n").collect::<Vec<&str>>()
    }

    #[test]
    fn match_reverse() {
        let lines = split("lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 3);
        assert_eq!(results.first().unwrap().hint, "a");
        assert_eq!(results.last().unwrap().hint, "c");
    }

    #[test]
    fn match_unique() {
        let lines = split("lorem 127.0.0.1 lorem 255.255.255.255 lorem 127.0.0.1 lorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(true);

        assert_eq!(results.len(), 3);
        assert_eq!(results.first().unwrap().hint, "a");
        assert_eq!(results.last().unwrap().hint, "a");
    }

    #[test]
    fn match_docker() {
        let lines = split("latest sha256:30557a29d5abc51e5f1d5b472e79b7e296f595abcf19fe6b9199dbbc809c6ff4 20 hours ago");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 1);
        assert_eq!(
            results.get(0).unwrap().text,
            "30557a29d5abc51e5f1d5b472e79b7e296f595abcf19fe6b9199dbbc809c6ff4"
        );
    }

    #[test]
    fn match_ansi_colors() {
        let lines = split("path: [32m/var/log/nginx.log[m\npath: [32mtest/log/nginx-2.log:32[mfolder/.nginx@4df2.log");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap().text, "/var/log/nginx.log");
        assert_eq!(results.get(1).unwrap().text, "test/log/nginx-2.log");
        assert_eq!(results.get(2).unwrap().text, "folder/.nginx@4df2.log");
    }

    #[test]
    fn match_paths() {
        let lines = split("Lorem /tmp/foo/bar_lol, lorem\n Lorem /var/log/boot-strap.log lorem ../log/kern.log lorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap().text, "/tmp/foo/bar_lol");
        assert_eq!(results.get(1).unwrap().text, "/var/log/boot-strap.log");
        assert_eq!(results.get(2).unwrap().text, "../log/kern.log");
    }

    #[test]
    fn match_home() {
        let lines = split("Lorem ~/.gnu/.config.txt, lorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0).unwrap().text, "~/.gnu/.config.txt");
    }

    #[test]
    fn match_uuids() {
        let lines =
            split("Lorem ipsum 123e4567-e89b-12d3-a456-426655440000 lorem\n Lorem lorem lorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn match_shas() {
        let lines = split("Lorem fd70b5695 5246ddf f924213 lorem\n Lorem 973113963b491874ab2e372ee60d4b4cb75f717c lorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 4);
        assert_eq!(results.get(0).unwrap().text, "fd70b5695");
        assert_eq!(results.get(1).unwrap().text, "5246ddf");
        assert_eq!(results.get(2).unwrap().text, "f924213");
        assert_eq!(
            results.get(3).unwrap().text,
            "973113963b491874ab2e372ee60d4b4cb75f717c"
        );
    }

    #[test]
    fn match_ips() {
        let lines =
            split("Lorem ipsum 127.0.0.1 lorem\n Lorem 255.255.10.255 lorem 127.0.0.1 lorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap().text, "127.0.0.1");
        assert_eq!(results.get(1).unwrap().text, "255.255.10.255");
        assert_eq!(results.get(2).unwrap().text, "127.0.0.1");
    }

    #[test]
    fn match_ipv6s() {
        let lines = split("Lorem ipsum fe80::2:202:fe4 lorem\n Lorem 2001:67c:670:202:7ba8:5e41:1591:d723 lorem fe80::2:1 lorem ipsum fe80:22:312:fe::1%eth0");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 4);
        assert_eq!(results.get(0).unwrap().text, "fe80::2:202:fe4");
        assert_eq!(
            results.get(1).unwrap().text,
            "2001:67c:670:202:7ba8:5e41:1591:d723"
        );
        assert_eq!(results.get(2).unwrap().text, "fe80::2:1");
        assert_eq!(results.get(3).unwrap().text, "fe80:22:312:fe::1%eth0");
    }

    #[test]
    fn match_markdown_urls() {
        let lines = split(
            "Lorem ipsum [link](https://github.io?foo=bar) ![](http://cdn.com/img.jpg) lorem",
        );
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().pattern, "markdown_url");
        assert_eq!(results.get(0).unwrap().text, "https://github.io?foo=bar");
        assert_eq!(results.get(1).unwrap().pattern, "markdown_url");
        assert_eq!(results.get(1).unwrap().text, "http://cdn.com/img.jpg");
    }

    #[test]
    fn match_urls() {
        let lines = split("Lorem ipsum https://www.rust-lang.org/tools lorem\n Lorem ipsumhttps://crates.io lorem https://github.io?foo=bar lorem ssh://github.io");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 4);
        assert_eq!(
            results.get(0).unwrap().text,
            "https://www.rust-lang.org/tools"
        );
        assert_eq!(results.get(0).unwrap().pattern, "url");
        assert_eq!(results.get(1).unwrap().text, "https://crates.io");
        assert_eq!(results.get(1).unwrap().pattern, "url");
        assert_eq!(results.get(2).unwrap().text, "https://github.io?foo=bar");
        assert_eq!(results.get(2).unwrap().pattern, "url");
        assert_eq!(results.get(3).unwrap().text, "ssh://github.io");
        assert_eq!(results.get(3).unwrap().pattern, "url");
    }

    #[test]
    fn match_addresses() {
        let lines = split("Lorem 0xfd70b5695 0x5246ddf lorem\n Lorem 0x973113tlorem");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap().text, "0xfd70b5695");
        assert_eq!(results.get(1).unwrap().text, "0x5246ddf");
        assert_eq!(results.get(2).unwrap().text, "0x973113");
    }

    #[test]
    fn match_hex_colors() {
        let lines =
            split("Lorem #fd7b56 lorem #FF00FF\n Lorem #00fF05 lorem #abcd00 lorem #afRR00");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 4);
        assert_eq!(results.get(0).unwrap().text, "#fd7b56");
        assert_eq!(results.get(1).unwrap().text, "#FF00FF");
        assert_eq!(results.get(2).unwrap().text, "#00fF05");
        assert_eq!(results.get(3).unwrap().text, "#abcd00");
    }

    #[test]
    fn match_ipfs() {
        let lines = split("Lorem QmRdbNSxDJBXmssAc9fvTtux4duptMvfSGiGuq6yHAQVKQ lorem Qmfoobar");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 1);
        assert_eq!(
            results.get(0).unwrap().text,
            "QmRdbNSxDJBXmssAc9fvTtux4duptMvfSGiGuq6yHAQVKQ"
        );
    }

    #[test]
    fn match_process_port() {
        let lines =
      split("Lorem 5695 52463 lorem\n Lorem 973113 lorem 99999 lorem 8888 lorem\n   23456 lorem 5432 lorem 23444");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 8);
    }

    #[test]
    fn match_diff_a() {
        let lines = split("Lorem lorem\n--- a/src/main.rs");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0).unwrap().text, "src/main.rs");
    }

    #[test]
    fn match_diff_b() {
        let lines = split("Lorem lorem\n+++ b/src/main.rs");
        let named_pat = vec![];
        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0).unwrap().text, "src/main.rs");
    }

    #[test]
    fn priority() {
        let lines = split("Lorem [link](http://foo.bar) ipsum CUSTOM-52463 lorem ISSUE-123 lorem\nLorem /var/fd70b569/9999.log 52463 lorem\n Lorem 973113 lorem 123e4567-e89b-12d3-a456-426655440000 lorem 8888 lorem\n  https://crates.io/23456/fd70b569 lorem");

        let named_pat = vec![];
        let custom: Vec<String> = ["CUSTOM-[0-9]{4,}", "ISSUE-[0-9]{3}"]
            .iter()
            .map(|&s| s.to_string())
            .collect();
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 9);
        assert_eq!(results.get(0).unwrap().text, "http://foo.bar");
        assert_eq!(results.get(1).unwrap().text, "CUSTOM-52463");
        assert_eq!(results.get(2).unwrap().text, "ISSUE-123");
        assert_eq!(results.get(3).unwrap().text, "/var/fd70b569/9999.log");
        assert_eq!(results.get(4).unwrap().text, "52463");
        assert_eq!(results.get(5).unwrap().text, "973113");
        assert_eq!(
            results.get(6).unwrap().text,
            "123e4567-e89b-12d3-a456-426655440000"
        );
        assert_eq!(results.get(7).unwrap().text, "8888");
        assert_eq!(
            results.get(8).unwrap().text,
            "https://crates.io/23456/fd70b569"
        );
    }

    #[test]
    fn named_patterns() {
        let lines = split("Lorem [link](http://foo.bar) ipsum CUSTOM-52463 lorem ISSUE-123 lorem\nLorem /var/fd70b569/9999.log 52463 lorem\n Lorem 973113 lorem 123e4567-e89b-12d3-a456-426655440000 lorem 8888 lorem\n  https://crates.io/23456/fd70b569 lorem");

        use crate::regexes::parse_pattern_name;
        let named_pat = vec![parse_pattern_name("url").unwrap()];

        let custom = vec![];
        let alphabet = Alphabet("abcd".to_string());
        let results = Model::new(&lines, &alphabet, &named_pat, &custom, false).matches(false);

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().text, "http://foo.bar");
        assert_eq!(
            results.get(1).unwrap().text,
            "https://crates.io/23456/fd70b569"
        );
    }
}
