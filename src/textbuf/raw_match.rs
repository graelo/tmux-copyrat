/// Internal surrogate for `Match`, before a Hint has been associated.
#[derive(Debug)]
pub(super) struct RawMatch<'a> {
    pub x: i32,
    pub y: i32,
    pub pattern: &'a str,
    pub text: &'a str,
}
