use std::char;
use std::io;
use std::str::FromStr;

use clap::Clap;
use sequence_trie::SequenceTrie;
use termion::{self, color, cursor, event, style};

use crate::error::ParseError;
use crate::{colors, model};

pub struct View<'a> {
    model: &'a mut model::Model<'a>,
    matches: Vec<model::Match<'a>>,
    lookup_trie: SequenceTrie<char, usize>,
    focus_index: usize,
    focus_wrap_around: bool,
    hint_alignment: &'a HintAlignment,
    rendering_colors: &'a ViewColors,
    hint_style: Option<HintStyle>,
}

impl<'a> View<'a> {
    pub fn new(
        model: &'a mut model::Model<'a>,
        unique_hint: bool,
        focus_wrap_around: bool,
        hint_alignment: &'a HintAlignment,
        rendering_colors: &'a ViewColors,
        hint_style: Option<HintStyle>,
    ) -> View<'a> {
        let matches = model.matches(unique_hint);
        let lookup_trie = model::Model::build_lookup_trie(&matches);
        let focus_index = if model.reverse { matches.len() - 1 } else { 0 };

        View {
            model,
            matches,
            lookup_trie,
            focus_index,
            focus_wrap_around,
            hint_alignment,
            rendering_colors,
            hint_style,
        }
    }

    /// Move focus onto the previous hint, returning both the index of the
    /// previously focused match, and the index of the newly focused one.
    fn prev_focus_index(&mut self) -> (usize, usize) {
        let old_index = self.focus_index;
        if self.focus_wrap_around {
            if self.focus_index == 0 {
                self.focus_index = self.matches.len() - 1;
            } else {
                self.focus_index -= 1;
            }
        } else {
            if self.focus_index > 0 {
                self.focus_index -= 1;
            }
        }
        let new_index = self.focus_index;
        (old_index, new_index)
    }

    /// Move focus onto the next hint, returning both the index of the
    /// previously focused match, and the index of the newly focused one.
    fn next_focus_index(&mut self) -> (usize, usize) {
        let old_index = self.focus_index;
        if self.focus_wrap_around {
            if self.focus_index == self.matches.len() - 1 {
                self.focus_index = 0;
            } else {
                self.focus_index += 1;
            }
        } else {
            if self.focus_index < self.matches.len() - 1 {
                self.focus_index += 1;
            }
        }
        let new_index = self.focus_index;
        (old_index, new_index)
    }

    /// Returns screen offset of a given `Match`.
    ///
    /// If multibyte characters occur before the hint (in the "prefix"), then
    /// their compouding takes less space on screen when printed: for
    /// instance ´ + e = é. Consequently the hint offset has to be adjusted
    /// to the left.
    fn match_offsets(&self, mat: &model::Match<'a>) -> (usize, usize) {
        let offset_x = {
            let line = &self.model.lines[mat.y as usize];
            let prefix = &line[0..mat.x as usize];
            let adjust = prefix.len() - prefix.chars().count();
            (mat.x as usize) - (adjust)
        };
        let offset_y = mat.y as usize;

        (offset_x, offset_y)
    }

    /// Render entire model lines on provided writer.
    ///
    /// This renders the basic content on which matches and hints can be rendered.
    ///
    /// # Notes
    /// - All trailing whitespaces are trimmed, empty lines are skipped.
    /// - This writes directly on the writer, avoiding extra allocation.
    fn render_base_text(stdout: &mut dyn io::Write, lines: &Vec<&str>, colors: &ViewColors) -> () {
        for (index, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim_end();

            if !trimmed_line.is_empty() {
                write!(
                    stdout,
                    "{goto}{bg_color}{fg_color}{text}{fg_reset}{bg_reset}",
                    goto = cursor::Goto(1, index as u16 + 1),
                    fg_color = color::Fg(colors.text_fg.as_ref()),
                    bg_color = color::Bg(colors.text_bg.as_ref()),
                    fg_reset = color::Fg(color::Reset),
                    bg_reset = color::Bg(color::Reset),
                    text = &trimmed_line,
                )
                .unwrap();
            }
        }
    }

    /// Render the Match's `text` field on provided writer using the `match_*g` color.
    ///
    /// If a Mach is "focused", it is then rendered with the `focused_*g` colors.
    ///
    /// # Note
    ///
    /// This writes directly on the writer, avoiding extra allocation.
    fn render_matched_text(
        stdout: &mut dyn io::Write,
        text: &str,
        focused: bool,
        offset: (usize, usize),
        colors: &ViewColors,
    ) {
        // To help identify it, the match thas has focus is rendered with a dedicated color.
        let (fg_color, bg_color) = if focused {
            (&colors.focused_fg, &colors.focused_bg)
        } else {
            (&colors.match_fg, &colors.match_bg)
        };

        // Render just the Match's text on top of existing content.
        write!(
            stdout,
            "{goto}{bg_color}{fg_color}{text}{fg_reset}{bg_reset}",
            goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1),
            fg_color = color::Fg(fg_color.as_ref()),
            bg_color = color::Bg(bg_color.as_ref()),
            fg_reset = color::Fg(color::Reset),
            bg_reset = color::Bg(color::Reset),
            text = &text,
        )
        .unwrap();
    }

    /// Render a Match's `hint` field on the provided writer.
    ///
    /// This renders the hint according to some provided style:
    /// - just colors
    /// - underlined with colors
    /// - surrounding the hint's text with some delimiters, see
    ///   `HintStyle::Delimited`.
    ///
    /// # Note
    /// This writes directly on the writer, avoiding extra allocation.
    fn render_matched_hint(
        stdout: &mut dyn io::Write,
        hint_text: &str,
        offset: (usize, usize),
        colors: &ViewColors,
        hint_style: &Option<HintStyle>,
    ) {
        let fg_color = color::Fg(colors.hint_fg.as_ref());
        let bg_color = color::Bg(colors.hint_bg.as_ref());
        let fg_reset = color::Fg(color::Reset);
        let bg_reset = color::Bg(color::Reset);

        match hint_style {
            None => {
                write!(
                    stdout,
                    "{goto}{bg_color}{fg_color}{hint}{fg_reset}{bg_reset}",
                    goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1),
                    fg_color = fg_color,
                    bg_color = bg_color,
                    fg_reset = fg_reset,
                    bg_reset = bg_reset,
                    hint = hint_text,
                )
                .unwrap();
            }
            Some(hint_style) => match hint_style {
                HintStyle::Bold => {
                    write!(
                        stdout,
                        "{goto}{bg_color}{fg_color}{sty}{hint}{sty_reset}{fg_reset}{bg_reset}",
                        goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1),
                        fg_color = fg_color,
                        bg_color = bg_color,
                        fg_reset = fg_reset,
                        bg_reset = bg_reset,
                        sty = style::Bold,
                        sty_reset = style::NoBold,
                        hint = hint_text,
                    )
                    .unwrap();
                }
                HintStyle::Italic => {
                    write!(
                        stdout,
                        "{goto}{bg_color}{fg_color}{sty}{hint}{sty_reset}{fg_reset}{bg_reset}",
                        goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1),
                        fg_color = fg_color,
                        bg_color = bg_color,
                        fg_reset = fg_reset,
                        bg_reset = bg_reset,
                        sty = style::Italic,
                        sty_reset = style::NoItalic,
                        hint = hint_text,
                    )
                    .unwrap();
                }
                HintStyle::Underline => {
                    write!(
                        stdout,
                        "{goto}{bg_color}{fg_color}{sty}{hint}{sty_reset}{fg_reset}{bg_reset}",
                        goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1),
                        fg_color = fg_color,
                        bg_color = bg_color,
                        fg_reset = fg_reset,
                        bg_reset = bg_reset,
                        sty = style::Underline,
                        sty_reset = style::NoUnderline,
                        hint = hint_text,
                    )
                    .unwrap();
                }
                HintStyle::Surround(opening, closing) => {
                    write!(
                        stdout,
                        "{goto}{bg_color}{fg_color}{bra}{hint}{bra_close}{fg_reset}{bg_reset}",
                        goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1),
                        fg_color = fg_color,
                        bg_color = bg_color,
                        fg_reset = fg_reset,
                        bg_reset = bg_reset,
                        bra = opening,
                        bra_close = closing,
                        hint = hint_text,
                    )
                    .unwrap();
                }
            },
        }
    }

    /// Convenience function that renders both the matched text and its hint,
    /// if focused.
    fn render_match(&self, stdout: &mut dyn io::Write, mat: &model::Match<'a>, focused: bool) {
        let text = mat.text;

        let (offset_x, offset_y) = self.match_offsets(mat);

        View::render_matched_text(
            stdout,
            text,
            focused,
            (offset_x, offset_y),
            &self.rendering_colors,
        );

        if !focused {
            // If not focused, render the hint (e.g. "eo") as an overlay on
            // top of the rendered matched text, aligned at its leading or the
            // trailing edge.
            let extra_offset = match self.hint_alignment {
                HintAlignment::Leading => 0,
                HintAlignment::Trailing => text.len() - mat.hint.len(),
            };

            View::render_matched_hint(
                stdout,
                &mat.hint,
                (offset_x + extra_offset, offset_y),
                &self.rendering_colors,
                &self.hint_style,
            );
        }
    }

    /// Full nender the view on the provided writer.
    ///
    /// This renders in 3 phases:
    /// - all lines are rendered verbatim
    /// - each Match's `text` is rendered as an overlay on top of it
    /// - each Match's `hint` text is rendered as a final overlay
    ///
    /// Depending on the value of `self.hint_alignment`, the hint can be rendered
    /// on the leading edge of the underlying Match's `text`,
    /// or on the trailing edge.
    ///
    /// # Note
    /// Multibyte characters are taken into account, so that the Match's `text`
    /// and `hint` are rendered in their proper position.
    fn full_render(&self, stdout: &mut dyn io::Write) -> () {
        // 1. Trim all lines and render non-empty ones.
        View::render_base_text(stdout, self.model.lines, &self.rendering_colors);

        for (index, mat) in self.matches.iter().enumerate() {
            let focused = index == self.focus_index;
            self.render_match(stdout, mat, focused);
        }

        stdout.flush().unwrap();
    }

    /// Render the previous match with its hint, and render the newly focused
    /// match without its hint. This is more efficient than a full render.
    fn diff_render(
        &self,
        stdout: &mut dyn io::Write,
        old_focus_index: usize,
        new_focus_index: usize,
    ) {
        // Render the previously focused match as non-focused
        let mat = self.matches.get(old_focus_index).unwrap();
        let focused = false;
        self.render_match(stdout, mat, focused);

        // Render the previously focused match as non-focused
        let mat = self.matches.get(new_focus_index).unwrap();
        let focused = true;
        self.render_match(stdout, mat, focused);

        stdout.flush().unwrap();
    }

    /// Listen to keys entered on stdin, moving focus accordingly, or
    /// selecting one match.
    ///
    /// # Panics
    ///
    /// - This function panics if termion cannot read the entered keys on stdin.
    fn listen(&mut self, reader: &mut dyn io::Read, writer: &mut dyn io::Write) -> Event {
        use termion::input::TermRead; // Trait for `reader.keys().next()`.

        if self.matches.is_empty() {
            return Event::Exit;
        }

        let mut typed_hint = String::new();

        self.full_render(writer);

        loop {
            // This is an option of a result of a key... Let's pop error cases first.
            let next_key = reader.keys().next();

            if next_key.is_none() {
                // Nothing in the buffer. Wait for a bit...
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }

            let key_res = next_key.unwrap();
            if let Err(err) = key_res {
                // Termion not being able to read from stdin is an unrecoverable error.
                panic!(err);
            }

            match key_res.unwrap() {
                event::Key::Esc => {
                    break;
                }

                // Move focus to next/prev match.
                event::Key::Up => {
                    let (old_index, focused_index) = self.prev_focus_index();
                    self.diff_render(writer, old_index, focused_index);
                }
                event::Key::Down => {
                    let (old_index, focused_index) = self.next_focus_index();
                    self.diff_render(writer, old_index, focused_index);
                }
                event::Key::Left => {
                    let (old_index, focused_index) = self.prev_focus_index();
                    self.diff_render(writer, old_index, focused_index);
                }
                event::Key::Right => {
                    let (old_index, focused_index) = self.next_focus_index();
                    self.diff_render(writer, old_index, focused_index);
                }
                event::Key::Char(_ch @ 'n') => {
                    let (old_index, focused_index) = if self.model.reverse {
                        self.prev_focus_index()
                    } else {
                        self.next_focus_index()
                    };
                    self.diff_render(writer, old_index, focused_index);
                }
                event::Key::Char(_ch @ 'N') => {
                    let (old_index, focused_index) = if self.model.reverse {
                        self.next_focus_index()
                    } else {
                        self.prev_focus_index()
                    };
                    self.diff_render(writer, old_index, focused_index);
                }

                // Yank/copy
                event::Key::Char(_ch @ 'y') => {
                    let text = self.matches.get(self.focus_index).unwrap().text;
                    return Event::Match((text.to_string(), false));
                }
                event::Key::Char(_ch @ 'Y') => {
                    let text = self.matches.get(self.focus_index).unwrap().text;
                    return Event::Match((text.to_string(), true));
                }

                // TODO: use a Trie or another data structure to determine
                // if the entered key belongs to a longer hint.
                // Attempts at finding a match with a corresponding hint.
                event::Key::Char(ch) => {
                    let key = ch.to_string();
                    let lower_key = key.to_lowercase();

                    typed_hint.push_str(&lower_key);

                    match self
                        .lookup_trie
                        .get_node(&typed_hint.chars().collect::<Vec<char>>())
                    {
                        None => {
                            // An unknown key was entered.
                            return Event::Exit;
                        }
                        Some(node) => {
                            if node.is_leaf() {
                                // The last key of a hint was entered.
                                let match_index = node.value().expect("By construction, the Lookup Trie should have a value for each leaf.");
                                let mat = self.matches.get(*match_index).expect("By construction, the value in a leaf should correspond to an existing hint.");
                                let text = mat.text.to_string();
                                let uppercased = key != lower_key;
                                return Event::Match((text, uppercased));
                            } else {
                                // The prefix of a hint was entered, but we
                                // still need more keys.
                                continue;
                            }
                        }
                    }
                }

                // Unknown keys are ignored.
                _ => (),
            }

            // End of event processing loop.
        }

        Event::Exit
    }

    /// Configure the terminal and display the `View`.
    ///
    /// - Setup steps: switch to alternate screen, switch to raw mode, hide the cursor.
    /// - Teardown steps: show cursor, back to main screen.
    pub fn present(&mut self) -> Option<(String, bool)> {
        use std::io::Write;
        use termion::raw::IntoRawMode;
        use termion::screen::AlternateScreen;

        let mut stdin = termion::async_stdin();
        let mut stdout = AlternateScreen::from(
            io::stdout()
                .into_raw_mode()
                .expect("Cannot access alternate screen."),
        );

        write!(stdout, "{}", cursor::Hide).unwrap();

        let selection = match self.listen(&mut stdin, &mut stdout) {
            Event::Exit => None,
            Event::Match((text, uppercased)) => Some((text, uppercased)),
        };

        write!(stdout, "{}", cursor::Show).unwrap();

        selection
    }
}

/// Describes if, during rendering, a hint should aligned to the leading edge of
/// the matched text, or to its trailing edge.
#[derive(Debug, Clap)]
pub enum HintAlignment {
    Leading,
    Trailing,
}

impl FromStr for HintAlignment {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<HintAlignment, ParseError> {
        match s {
            "leading" => Ok(HintAlignment::Leading),
            "trailing" => Ok(HintAlignment::Trailing),
            _ => Err(ParseError::ExpectedString(String::from(
                "leading or trailing",
            ))),
        }
    }
}

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

/// Returned value after the `View` has finished listening to events.
enum Event {
    /// Exit with no selected matches,
    Exit,
    /// A vector of matched text and whether it was selected with uppercase.
    Match((String, bool)),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabets;

    #[test]
    fn test_render_all_lines() {
        let content = "some text
* e006b06 - (12 days ago) swapper: Make quotes
path: /usr/local/bin/git


path: /usr/local/bin/cargo";
        let lines: Vec<&str> = content.split('\n').collect();
        let colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        let mut writer = vec![];
        View::render_base_text(&mut writer, &lines, &colors);

        let goto1 = cursor::Goto(1, 1);
        let goto2 = cursor::Goto(1, 2);
        let goto3 = cursor::Goto(1, 3);
        let goto6 = cursor::Goto(1, 6);
        assert_eq!(
            writer,
            format!(
                "{g1}{bg}{fg}some text{fg_reset}{bg_reset}{g2}{bg}{fg}* e006b06 - (12 days ago) swapper: Make quotes{fg_reset}{bg_reset}{g3}{bg}{fg}path: /usr/local/bin/git{fg_reset}{bg_reset}{g6}{bg}{fg}path: /usr/local/bin/cargo{fg_reset}{bg_reset}",
                g1 = goto1, g2 = goto2, g3 = goto3, g6 = goto6,
                fg = color::Fg(colors.text_fg.as_ref()),
                bg = color::Bg(colors.text_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                )
            .as_bytes()
            );
    }

    #[test]
    fn test_render_focused_matched_text() {
        let mut writer = vec![];
        let text = "https://en.wikipedia.org/wiki/Barcelona";
        let focused = true;
        let offset: (usize, usize) = (3, 1);
        let colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        View::render_matched_text(&mut writer, text, focused, offset, &colors);

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{text}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.focused_fg.as_ref()),
                bg = color::Bg(colors.focused_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                text = &text,
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_render_matched_text() {
        let mut writer = vec![];
        let text = "https://en.wikipedia.org/wiki/Barcelona";
        let focused = false;
        let offset: (usize, usize) = (3, 1);
        let colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        View::render_matched_text(&mut writer, text, focused, offset, &colors);

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{text}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.match_fg.as_ref()),
                bg = color::Bg(colors.match_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                text = &text,
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_render_unstyled_matched_hint() {
        let mut writer = vec![];
        let hint_text = "eo";
        let offset: (usize, usize) = (3, 1);
        let colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        let extra_offset = 0;
        let hint_style = None;

        View::render_matched_hint(
            &mut writer,
            hint_text,
            (offset.0 + extra_offset, offset.1),
            &colors,
            &hint_style,
        );

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{text}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.hint_fg.as_ref()),
                bg = color::Bg(colors.hint_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                text = "eo",
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_render_underlined_matched_hint() {
        let mut writer = vec![];
        let hint_text = "eo";
        let offset: (usize, usize) = (3, 1);
        let colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        let extra_offset = 0;
        let hint_style = Some(HintStyle::Underline);

        View::render_matched_hint(
            &mut writer,
            hint_text,
            (offset.0 + extra_offset, offset.1),
            &colors,
            &hint_style,
        );

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{sty}{text}{sty_reset}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.hint_fg.as_ref()),
                bg = color::Bg(colors.hint_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                sty = style::Underline,
                sty_reset = style::NoUnderline,
                text = "eo",
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_render_bracketed_matched_hint() {
        let mut writer = vec![];
        let hint_text = "eo";
        let offset: (usize, usize) = (3, 1);
        let colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        let extra_offset = 0;
        let hint_style = Some(HintStyle::Surround('{', '}'));

        View::render_matched_hint(
            &mut writer,
            hint_text,
            (offset.0 + extra_offset, offset.1),
            &colors,
            &hint_style,
        );

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{bra}{text}{bra_close}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.hint_fg.as_ref()),
                bg = color::Bg(colors.hint_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                bra = '{',
                bra_close = '}',
                text = "eo",
            )
            .as_bytes()
        );
    }

    #[test]
    /// Simulates rendering without any match.
    fn test_render_full_without_matches() {
        let content = "lorem 127.0.0.1 lorem

Barcelona https://en.wikipedia.org/wiki/Barcelona -   ";

        let lines = content.split('\n').collect();

        let named_pat = vec![];
        let custom_regexes = vec![];
        let alphabet = alphabets::Alphabet("abcd".to_string());
        let mut model = model::Model::new(&lines, &alphabet, &named_pat, &custom_regexes, false);
        let rendering_colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };
        let hint_alignment = HintAlignment::Leading;

        // create a view without any match
        let view = View {
            model: &mut model,
            matches: vec![], // no matches
            lookup_trie: SequenceTrie::new(),
            focus_index: 0,
            focus_wrap_around: false,
            hint_alignment: &hint_alignment,
            rendering_colors: &rendering_colors,
            hint_style: None,
        };

        let mut writer = vec![];
        view.full_render(&mut writer);

        let goto1 = cursor::Goto(1, 1);
        let goto3 = cursor::Goto(1, 3);

        let expected = format!(
            "{goto1}{bg}{fg}lorem 127.0.0.1 lorem{fg_reset}{bg_reset}\
        {goto3}{bg}{fg}Barcelona https://en.wikipedia.org/wiki/Barcelona -{fg_reset}{bg_reset}",
            goto1 = goto1,
            goto3 = goto3,
            fg = color::Fg(rendering_colors.text_fg.as_ref()),
            bg = color::Bg(rendering_colors.text_bg.as_ref()),
            fg_reset = color::Fg(color::Reset),
            bg_reset = color::Bg(color::Reset),
        );

        // println!("{:?}", writer);
        // println!("{:?}", expected.as_bytes());

        // println!("matches: {}", view.matches.len());
        // println!("lines: {}", lines.len());

        assert_eq!(writer, expected.as_bytes());
    }

    #[test]
    /// Simulates rendering with matches.
    fn test_render_full_with_matches() {
        let content = "lorem 127.0.0.1 lorem

Barcelona https://en.wikipedia.org/wiki/Barcelona -   ";

        let lines = content.split('\n').collect();

        let named_pat = vec![];
        let custom_regexes = vec![];
        let alphabet = alphabets::Alphabet("abcd".to_string());
        let reverse = true;
        let mut model = model::Model::new(&lines, &alphabet, &named_pat, &custom_regexes, reverse);
        let unique_hint = false;
        let wrap_around = false;

        let rendering_colors = ViewColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };
        let hint_alignment = HintAlignment::Leading;
        let hint_style = None;

        let view = View::new(
            &mut model,
            unique_hint,
            wrap_around,
            &hint_alignment,
            &rendering_colors,
            hint_style,
        );

        let mut writer = vec![];
        view.full_render(&mut writer);

        let expected_content = {
            let goto1 = cursor::Goto(1, 1);
            let goto3 = cursor::Goto(1, 3);

            format!(
                "{goto1}{bg}{fg}lorem 127.0.0.1 lorem{fg_reset}{bg_reset}\
        {goto3}{bg}{fg}Barcelona https://en.wikipedia.org/wiki/Barcelona -{fg_reset}{bg_reset}",
                goto1 = goto1,
                goto3 = goto3,
                fg = color::Fg(rendering_colors.text_fg.as_ref()),
                bg = color::Bg(rendering_colors.text_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset)
            )
        };

        let expected_match1_text = {
            let goto7_1 = cursor::Goto(7, 1);
            format!(
                "{goto7_1}{match_bg}{match_fg}127.0.0.1{fg_reset}{bg_reset}",
                goto7_1 = goto7_1,
                match_fg = color::Fg(rendering_colors.match_fg.as_ref()),
                match_bg = color::Bg(rendering_colors.match_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset)
            )
        };

        let expected_match1_hint = {
            let goto7_1 = cursor::Goto(7, 1);

            format!(
                "{goto7_1}{hint_bg}{hint_fg}b{fg_reset}{bg_reset}",
                goto7_1 = goto7_1,
                hint_fg = color::Fg(rendering_colors.hint_fg.as_ref()),
                hint_bg = color::Bg(rendering_colors.hint_bg.as_ref()),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset)
            )
        };

        let expected_match2_text = {
            let goto11_3 = cursor::Goto(11, 3);
            format!(
        "{goto11_3}{focus_bg}{focus_fg}https://en.wikipedia.org/wiki/Barcelona{fg_reset}{bg_reset}",
        goto11_3 = goto11_3,
        focus_fg = color::Fg(rendering_colors.focused_fg.as_ref()),
        focus_bg = color::Bg(rendering_colors.focused_bg.as_ref()),
        fg_reset = color::Fg(color::Reset),
        bg_reset = color::Bg(color::Reset)
      )
        };

        // Because reverse is true, this second match is focused,
        // then the hint should not be rendered.

        // let expected_match2_hint = {
        //     let goto11_3 = cursor::Goto(11, 3);

        //     format!(
        //         "{goto11_3}{hint_bg}{hint_fg}a{fg_reset}{bg_reset}",
        //         goto11_3 = goto11_3,
        //         hint_fg = color::Fg(rendering_colors.hint_fg.as_ref()),
        //         hint_bg = color::Bg(rendering_colors.hint_bg.as_ref()),
        //         fg_reset = color::Fg(color::Reset),
        //         bg_reset = color::Bg(color::Reset)
        //     )
        // };

        let expected = [
            expected_content,
            expected_match1_text,
            expected_match1_hint,
            expected_match2_text,
            // expected_match2_hint,
        ]
        .concat();

        // println!("{:?}", writer);
        // println!("{:?}", expected.as_bytes());

        // let diff_point = writer
        //   .iter()
        //   .zip(expected.as_bytes().iter())
        //   .enumerate()
        //   .find(|(_idx, (&l, &r))| l != r);
        // println!("{:?}", diff_point);

        assert_eq!(2, view.matches.len());

        assert_eq!(writer, expected.as_bytes());
    }
}

/// Holds color-related data, for clarity.
///
/// - `focus_*` colors are used to render the currently focused matched text.
/// - `normal_*` colors are used to render other matched text.
/// - `hint_*` colors are used to render the hints.
#[derive(Clap, Debug)]
pub struct ViewColors {
    /// Foreground color for base text.
    #[clap(long, default_value = "bright-cyan", parse(try_from_str = colors::parse_color))]
    pub text_fg: Box<dyn color::Color>,

    /// Background color for base text.
    #[clap(long, default_value = "bright-white", parse(try_from_str = colors::parse_color))]
    pub text_bg: Box<dyn color::Color>,

    /// Foreground color for matches.
    #[clap(long, default_value = "yellow",
                parse(try_from_str = colors::parse_color))]
    pub match_fg: Box<dyn color::Color>,

    /// Background color for matches.
    #[clap(long, default_value = "bright-white",
                parse(try_from_str = colors::parse_color))]
    pub match_bg: Box<dyn color::Color>,

    /// Foreground color for the focused match.
    #[clap(long, default_value = "magenta",
                parse(try_from_str = colors::parse_color))]
    pub focused_fg: Box<dyn color::Color>,

    /// Background color for the focused match.
    #[clap(long, default_value = "bright-white",
                parse(try_from_str = colors::parse_color))]
    pub focused_bg: Box<dyn color::Color>,

    /// Foreground color for hints.
    #[clap(long, default_value = "white",
                parse(try_from_str = colors::parse_color))]
    pub hint_fg: Box<dyn color::Color>,

    /// Background color for hints.
    #[clap(long, default_value = "magenta",
                parse(try_from_str = colors::parse_color))]
    pub hint_bg: Box<dyn color::Color>,
}
