use crate::error;

/// Catalog of available alphabets.
///
/// # Note
///
/// Keep in mind letters 'n' and 'y' are systematically removed at runtime to
/// prevent conflict with navigation and yank/copy keys.
const ALPHABETS: [(&'static str, &'static str); 21] = [
    // ("abcd", "abcd"),
    ("qwerty", "asdfqwerzxcvjklmiuopghtybn"),
    ("qwerty-homerow", "asdfjklgh"),
    ("qwerty-left-hand", "asdfqwerzcxv"),
    ("qwerty-right-hand", "jkluiopmyhn"),
    ("azerty", "qsdfazerwxcvjklmuiopghtybn"),
    ("azerty-homerow", "qsdfjkmgh"),
    ("azerty-left-hand", "qsdfazerwxcv"),
    ("azerty-right-hand", "jklmuiophyn"),
    ("qwertz", "asdfqweryxcvjkluiopmghtzbn"),
    ("qwertz-homerow", "asdfghjkl"),
    ("qwertz-left-hand", "asdfqweryxcv"),
    ("qwertz-right-hand", "jkluiopmhzn"),
    ("dvorak", "aoeuqjkxpyhtnsgcrlmwvzfidb"),
    ("dvorak-homerow", "aoeuhtnsid"),
    ("dvorak-left-hand", "aoeupqjkyix"),
    ("dvorak-right-hand", "htnsgcrlmwvz"),
    ("colemak", "arstqwfpzxcvneioluymdhgjbk"),
    ("colemak-homerow", "arstneiodh"),
    ("colemak-left-hand", "arstqwfpzxcv"),
    ("colemak-right-hand", "neioluymjhk"),
    (
        "longest",
        "aoeuqjkxpyhtnsgcrlmwvzfidb-;,~<>'@!#$%^&*~1234567890",
    ),
];

/// Parse a name string into `Alphabet`, used during CLI parsing.
///
/// # Note
///
/// Letters 'n' and 'N' are systematically removed to prevent conflict with
/// navigation keys (arrows and 'n' 'N'). Letters 'y' and 'Y' are also removed
/// to prevent conflict with yank/copy.
pub fn parse_alphabet(src: &str) -> Result<Alphabet, error::ParseError> {
    let alphabet_pair = ALPHABETS.iter().find(|&(name, _letters)| name == &src);

    match alphabet_pair {
        Some((_name, letters)) => {
            let letters = letters.replace(&['n', 'N', 'y', 'Y'][..], "");
            Ok(Alphabet(letters.to_string()))
        }
        None => Err(error::ParseError::UnknownAlphabet),
    }
}

/// Type-safe string alphabet (newtype).
#[derive(Debug)]
pub struct Alphabet(pub String);

impl Alphabet {
    /// Create `n` hints from the Alphabet.
    ///
    /// An Alphabet of `m` letters can produce at most `m^2` hints. In case
    /// this limit is exceeded, this function will generate the `n` hints from
    /// an Alphabet which has more letters (50). This will ensure 2500 hints
    /// can be generated, which should cover all use cases (I think even
    /// easymotion has less).
    ///
    /// If more hints are needed, unfortunately, this will keep producing
    /// empty (`""`) hints.
    ///
    /// ```
    /// // The algorithm works as follows:
    /// //                                  --- lead ----
    /// // initial state                 |  a   b   c   d
    ///
    /// // along as we need more hints, and still have capacity, do the following
    ///
    /// //                                  --- lead ----  --- gen --- -------------- prev ---------------
    /// // pick d, generate da db dc dd  |  a   b   c  (d) da db dc dd
    /// // pick c, generate ca cb cc cd  |  a   b  (c) (d) ca cb cc cd da db dc dd
    /// // pick b, generate ba bb bc bd  |  a  (b) (c) (d) ba bb bc bd ca cb cc cd da db dc dd
    /// // pick a, generate aa ab ac ad  | (a) (b) (c) (d) aa ab ac ad ba bb bc bd ca cb cc cd da db dc dd
    /// ```
    pub fn make_hints(&self, n: usize) -> Vec<String> {
        // Shortcut if we have enough letters in the Alphabet.
        if self.0.len() >= n {
            return self.0.chars().take(n).map(|c| c.to_string()).collect();
        }

        // Use the "longest" alphabet if the current alphabet cannot produce as
        // many hints as asked.
        let letters: Vec<char> = if self.0.len().pow(2) >= n {
            self.0.chars().collect()
        } else {
            let alt_alphabet = parse_alphabet("longest").unwrap();
            alt_alphabet.0.chars().collect()
        };

        let mut lead = letters.clone();
        let mut prev: Vec<String> = Vec::new();

        loop {
            if lead.len() + prev.len() >= n {
                break;
            }

            if lead.is_empty() {
                break;
            }
            let prefix = lead.pop().unwrap();

            // generate characters pairs
            let gen: Vec<String> = letters
                .iter()
                .take(n - lead.len() - prev.len())
                .map(|c| format!("{}{}", prefix, c))
                .collect();

            // Insert gen in front of prev
            prev.splice(..0, gen);
        }

        // Finalize by concatenating the lead and prev components, filling
        // with "" as necessary.
        let lead: Vec<String> = lead.iter().map(|c| c.to_string()).collect();

        let filler: Vec<String> = std::iter::repeat("")
            .take(n - lead.len() - prev.len())
            .map(|s| s.to_string())
            .collect();

        [lead, prev, filler].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_matches() {
        let alphabet = Alphabet("abcd".to_string());
        let hints = alphabet.make_hints(3);
        assert_eq!(hints, ["a", "b", "c"]);
    }

    #[test]
    fn composed_matches() {
        let alphabet = Alphabet("abcd".to_string());
        let hints = alphabet.make_hints(6);
        assert_eq!(hints, ["a", "b", "c", "da", "db", "dc"]);
    }

    #[test]
    fn composed_matches_multiple() {
        let alphabet = Alphabet("abcd".to_string());
        let hints = alphabet.make_hints(8);
        assert_eq!(hints, ["a", "b", "ca", "cb", "da", "db", "dc", "dd"]);
    }

    #[test]
    fn composed_matches_max_2() {
        let alphabet = Alphabet("ab".to_string());
        let hints = alphabet.make_hints(4);
        assert_eq!(hints, ["aa", "ab", "ba", "bb"]);
    }

    #[test]
    fn composed_matches_max_4() {
        let alphabet = Alphabet("abcd".to_string());
        let hints = alphabet.make_hints(13);
        assert_eq!(
            hints,
            ["a", "ba", "bb", "bc", "bd", "ca", "cb", "cc", "cd", "da", "db", "dc", "dd"]
        );
    }

    #[test]
    fn matches_with_longest_alphabet() {
        let alphabet = Alphabet("ab".to_string());
        let hints = alphabet.make_hints(2500);
        assert_eq!(hints.len(), 2500);
        assert_eq!(&hints[..3], ["aa", "ao", "ae"]);
        assert_eq!(&hints[2497..], ["08", "09", "00"]);
    }

    #[test]
    fn matches_exceed_longest_alphabet() {
        let alphabet = Alphabet("ab".to_string());
        let hints = alphabet.make_hints(10000);
        // 2500 unique hints are produced from the longest alphabet
        // The 7500 last ones come from the filler ("" empty hints).
        assert_eq!(hints.len(), 10000);
        assert!(&hints[2500..].iter().all(|s| s == ""));
    }
}
