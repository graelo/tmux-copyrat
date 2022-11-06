use std::collections;

use regex::Regex;
use sequence_trie::SequenceTrie;

use super::alphabet::Alphabet;
use super::raw_span::RawSpan;
use super::regexes::{NamedPattern, EXCLUDE_PATTERNS, PATTERNS};
use super::span::Span;

/// Holds data for the `Ui`.
pub struct Model<'a> {
    pub lines: &'a [&'a str],
    pub reverse: bool,
    pub spans: Vec<Span<'a>>,
    pub lookup_trie: SequenceTrie<char, usize>,
}

impl<'a> Model<'a> {
    pub fn new(
        lines: &'a [&'a str],
        alphabet: &'a Alphabet,
        use_all_patterns: bool,
        named_patterns: &'a [NamedPattern],
        custom_patterns: &'a [String],
        reverse: bool,
        unique_hint: bool,
    ) -> Model<'a> {
        let mut raw_spans =
            find_raw_spans(lines, named_patterns, custom_patterns, use_all_patterns);

        if reverse {
            raw_spans.reverse();
        }

        let mut spans = associate_hints(&raw_spans, alphabet, unique_hint);

        if reverse {
            spans.reverse();
        }

        let lookup_trie = build_lookup_trie(&spans);

        Model {
            // buffer,
            lines,
            reverse,
            spans,
            lookup_trie,
        }
    }
}

/// Internal function that searches the model's lines for pattern matches.
/// Returns a vector of `RawSpan` (text, location, pattern id) without
/// an associated hint. The hint is attached to `Span`, not to `RawSpan`.
///
/// # Notes
///
/// Custom regexes have priority over other regexes.
///
/// If no named patterns were specified, it will search for all available
/// patterns from the `PATTERNS` catalog.
fn find_raw_spans<'a>(
    lines: &'a [&'a str],
    named_patterns: &'a [NamedPattern],
    custom_patterns: &'a [String],
    use_all_patterns: bool,
) -> Vec<RawSpan<'a>> {
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

    let mut raw_spans = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        // Chunk is the remainder of the line to be searched for matches.
        // This advances iteratively, until no matches can be found.
        let mut chunk: &str = line;
        let mut offset: i32 = 0;

        // Use all avail regexes to match the chunk and select the match
        // occuring the earliest on the chunk. Save its matched text and
        // position in a `RawSpan` struct.
        loop {
            // For each avalable regex, use the `find_iter` iterator to
            // get the first non-overlapping match in the chunk, returning
            // the start and end byte indices with respect to the chunk.
            let chunk_matches = all_regexes
                .iter()
                .filter_map(|(&ref pat_name, reg)| {
                    reg.find_iter(chunk)
                        .next()
                        .map(|reg_match| (pat_name, reg, reg_match))
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

                // All patterns must have a capturing group: try obtaining
                // that text and start offset.
                let capture = reg
                    .captures_iter(text)
                    .next()
                    .expect("This regex is guaranteed to match.")
                    .get(1)
                    .expect("This regex should have a capture group.");

                let (subtext, substart) = (capture.as_str(), capture.start());

                raw_spans.push(RawSpan {
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

    raw_spans
}

/// Associate a hint to each `RawSpan`, returning a vector of `Span`.
///
/// If `unique` is `true`, all duplicate spans will have the same hint.
/// For copying text spans, this seems easier and more natural.
/// If `unique` is `false`, duplicate spans will have their own hint.
fn associate_hints<'a>(
    raw_spans: &[RawSpan<'a>],
    alphabet: &'a Alphabet,
    unique: bool,
) -> Vec<Span<'a>> {
    let hints = alphabet.make_hints(raw_spans.len());
    let mut hints_iter = hints.iter();

    let mut result: Vec<Span<'a>> = vec![];

    if unique {
        // Map (text, hint)
        let mut known: collections::HashMap<&str, &str> = collections::HashMap::new();

        for raw_span in raw_spans {
            let hint: &str = known.entry(raw_span.text).or_insert_with(|| {
                hints_iter
                    .next()
                    .expect("We should have as many hints as necessary, even invisible ones.")
            });

            result.push(Span {
                x: raw_span.x,
                y: raw_span.y,
                pattern: raw_span.pattern,
                text: raw_span.text,
                hint: hint.to_string(),
            });
        }
    } else {
        for raw_span in raw_spans {
            let hint = hints_iter
                .next()
                .expect("We should have as many hints as necessary, even invisible ones.");

            result.push(Span {
                x: raw_span.x,
                y: raw_span.y,
                pattern: raw_span.pattern,
                text: raw_span.text,
                hint: hint.to_string(),
            });
        }
    }

    result
}

/// Builds a `SequenceTrie` that helps determine if a sequence of keys
/// entered by the user corresponds to a match. This kind of lookup
/// directly returns a reference to the corresponding `Span` if any.
fn build_lookup_trie<'a>(spans: &'a [Span<'a>]) -> SequenceTrie<char, usize> {
    let mut trie = SequenceTrie::new();

    for (index, span) in spans.iter().enumerate() {
        let hint_chars = span.hint.chars().collect::<Vec<char>>();

        // no need to insert twice the same hint
        if trie.get(&hint_chars).is_none() {
            trie.insert_owned(hint_chars, index);
        }
    }

    trie
}
