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
    pub lines: &'a [&'a str],
    pub reverse: bool,
    pub matches: Vec<Match<'a>>,
    pub lookup_trie: SequenceTrie<char, usize>,
}

impl<'a> Model<'a> {
    pub fn new(
        // buffer: &'a str,
        lines: &'a [&'a str],
        alphabet: &'a Alphabet,
        use_all_patterns: bool,
        named_patterns: &'a [NamedPattern],
        custom_patterns: &'a [String],
        reverse: bool,
        unique_hint: bool,
    ) -> Model<'a> {
        // let lines = buffer.split('\n').collect::<Vec<_>>();

        let mut raw_matches =
            raw_matches(&lines, named_patterns, custom_patterns, use_all_patterns);

        if reverse {
            raw_matches.reverse();
        }

        let mut matches = associate_hints(&raw_matches, alphabet, unique_hint);

        if reverse {
            matches.reverse();
        }

        let lookup_trie = build_lookup_trie(&matches);

        Model {
            // buffer,
            lines,
            reverse,
            matches,
            lookup_trie,
        }
    }
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
fn raw_matches<'a>(
    lines: &'a [&'a str],
    named_patterns: &'a [NamedPattern],
    custom_patterns: &'a [String],
    use_all_patterns: bool,
) -> Vec<RawMatch<'a>> {
    let exclude_regexes = EXCLUDE_PATTERNS
        .iter()
        .map(|&(name, pattern)| (name, Regex::new(pattern).unwrap()))
        .collect::<Vec<_>>();

    let custom_regexes = custom_patterns
        .iter()
        .map(|pattern| {
            (
                "custom",
                Regex::new(pattern).expect("Invalid custom regexp"),
            )
        })
        .collect::<Vec<_>>();

    let regexes = if use_all_patterns {
        PATTERNS
            .iter()
            .map(|&(name, pattern)| (name, Regex::new(pattern).unwrap()))
            .collect::<Vec<(&str, regex::Regex)>>()
    } else {
        named_patterns
            .iter()
            .map(|NamedPattern(name, pattern)| (name.as_str(), Regex::new(pattern).unwrap()))
            .collect::<Vec<(&str, regex::Regex)>>()
    };

    let all_regexes = [exclude_regexes, custom_regexes, regexes].concat();

    let mut raw_matches = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        // Chunk is the remainder of the line to be searched for matches.
        // This advances iteratively, until no matches can be found.
        let mut chunk: &str = line;
        let mut offset: i32 = 0;

        // Use all avail regexes to match the chunk and select the match
        // occuring the earliest on the chunk. Save its matched text and
        // position in a `RawMatch` struct.
        loop {
            // For each avalable regex, use the `find_iter` iterator to
            // get the first non-overlapping match in the chunk, returning
            // the start and end byte indices with respect to the chunk.
            let chunk_matches = all_regexes
                .iter()
                .filter_map(|(&ref pat_name, reg)| match reg.find_iter(chunk).next() {
                    Some(reg_match) => Some((pat_name, reg, reg_match)),
                    None => None,
                })
                .collect::<Vec<_>>();

            if chunk_matches.is_empty() {
                break;
            }

            // First match on the chunk.
            let (pat_name, reg, reg_match) = chunk_matches
                .iter()
                .min_by_key(|element| element.2.start())
                .unwrap();

            // Never hint or break ansi color sequences.
            if *pat_name != "ansi_colors" {
                let text = reg_match.as_str();

                // In case the pattern has a capturing group, try obtaining
                // that text and start offset, else use the entire match.
                let (subtext, substart) = match reg
                    .captures_iter(text)
                    .next()
                    .expect("This regex is guaranteed to match.")
                    .get(1)
                {
                    Some(capture) => (capture.as_str(), capture.start()),
                    None => (text, 0),
                };

                raw_matches.push(RawMatch {
                    x: offset + reg_match.start() as i32 + substart as i32,
                    y: index as i32,
                    pattern: pat_name,
                    text: subtext,
                });
            }

            chunk = chunk
                .get(reg_match.end()..)
                .expect("The chunk must be larger than the regex match.");
            offset += reg_match.end() as i32;
        }
    }

    raw_matches
}

/// Associate a hint to each `RawMatch`, returning a vector of `Match`es.
///
/// If `unique` is `true`, all duplicate matches will have the same hint.
/// For copying matched text, this seems easier and more natural.
/// If `unique` is `false`, duplicate matches will have their own hint.
fn associate_hints<'a>(
    raw_matches: &[RawMatch<'a>],
    alphabet: &'a Alphabet,
    unique: bool,
) -> Vec<Match<'a>> {
    let hints = alphabet.make_hints(raw_matches.len());
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
fn build_lookup_trie<'a>(matches: &'a [Match<'a>]) -> SequenceTrie<char, usize> {
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
