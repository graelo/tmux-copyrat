/// Describes the style of contrast to be used during rendering of the hint's
/// text.
///
/// # Note
/// In practice, this is wrapped in an `Option`, so that the hint's text can be rendered with no style.
pub enum HintStyle {
    /// The hint's text will be bold (leveraging `termion::style::Bold`).
    Bold,
    /// The hint's text will be italicized (leveraging `termion::style::Italic`).
    Italic,
    /// The hint's text will be underlined (leveraging `termion::style::Underline`).
    Underline,
    /// The hint's text will be surrounded by these chars.
    Surround(char, char),
}
