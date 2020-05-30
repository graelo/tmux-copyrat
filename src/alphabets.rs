use crate::error;

const ALPHABETS: [(&'static str, &'static str); 22] = [
    ("numeric", "1234567890"),
    ("abcd", "abcd"),
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
];

/// Type-safe string alphabet (newtype).
#[derive(Debug)]
pub struct Alphabet(pub String);

impl Alphabet {
    /// Create `n` hints.
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
        let letters: Vec<String> = self.0.chars().map(|s| s.to_string()).collect();

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
                .map(|s| prefix.clone() + s)
                .collect();

            // Insert gen in front of prev
            prev.splice(0..0, gen);
        }

        lead = lead.iter().take(n - prev.len()).cloned().collect();
        lead.append(&mut prev);
        lead
    }
}

/// Parse a name string into `Alphabet`, supporting the CLI.
///
/// # Note
///
/// Letters 'n' and 'N' are systematically removed to prevent conflict with
/// navigation keys (arrows and 'n' 'N').
pub fn parse_alphabet(src: &str) -> Result<Alphabet, error::ParseError> {
    let alphabet_pair = ALPHABETS.iter().find(|&(name, _letters)| name == &src);

    match alphabet_pair {
        Some((_name, letters)) => {
            let letters = letters.replace(&['n', 'N'][..], "");
            Ok(Alphabet(letters.to_string()))
        }
        None => Err(error::ParseError::UnknownAlphabet),
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
        let hints = alphabet.make_hints(8);
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
        // a (b) (c) (d) a ba bc bd ca cb cc cd da db dc dd
    }
}
