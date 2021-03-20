use std::collections;

use regex::Regex;
use sequence_trie::SequenceTrie;

use super::alphabet::Alphabet;
use super::matches::Match;
use super::raw_match::RawMatch;
use super::regexes::{NamedPattern, EXCLUDE_PATTERNS, PATTERNS};

/// Holds data for the `Ui`.
pub struct Model<'a> {
    // buffer: &'a str,
    pub lines: Vec<&'a str>,
    alphabet: &'a Alphabet,
    use_all_patterns: bool,
    named_patterns: &'a [NamedPattern],
    custom_patterns: &'a [String],
    pub reverse: bool,
}

impl<'a> Model<'a> {
    pub fn new(
        buffer: &'a str,
        alphabet: &'a Alphabet,
        use_all_patterns: bool,
        named_patterns: &'a [NamedPattern],
        custom_patterns: &'a [String],
        reverse: bool,
    ) -> Model<'a> {
        let lines = buffer.split('\n').collect();

        Model {
            // buffer,
            lines,
            alphabet,
            use_all_patterns,
            named_patterns,
            custom_patterns,
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
            .custom_patterns
            .iter()
            .map(|pattern| {
                (
                    "custom",
                    Regex::new(pattern).expect("Invalid custom regexp"),
                )
            })
            .collect::<Vec<_>>();

        let regexes = if self.use_all_patterns {
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
                    .filter_map(|(&ref name, regex)| match regex.find_iter(chunk).next() {
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
    fn associate_hints(&self, raw_matches: &[RawMatch<'a>], unique: bool) -> Vec<Match<'a>> {
        let hints = self.alphabet.make_hints(raw_matches.len());
        let mut hints_iter = hints.iter();

        let mut result: Vec<Match<'a>> = vec![];

        if unique {
            // Map (text, hint)
            let mut known: collections::HashMap<&str, &str> = collections::HashMap::new();

            for raw_mat in raw_matches {
                let hint: &str = known.entry(raw_mat.text).or_insert_with(|| {
                    hints_iter
                        .next()
                        .expect("We should have as many hints as necessary, even invisible ones.")
                });

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
    pub fn build_lookup_trie(matches: &'a [Match<'a>]) -> SequenceTrie<char, usize> {
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
