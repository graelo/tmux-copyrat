/// Represents some span of text, its location on screen, the pattern that
/// created it, and the associated hint.
#[derive(Debug)]
pub struct Span<'a> {
    pub x: i32,
    pub y: i32,
    pub pattern: &'a str,
    pub text: &'a str,
    pub hint: String,
}
