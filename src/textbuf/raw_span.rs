/// Internal surrogate for `Span`, before a Hint has been associated.
#[derive(Debug)]
pub(super) struct RawSpan<'a> {
    pub x: i32,
    pub y: i32,
    pub pattern: &'a str,
    pub text: &'a str,
}
