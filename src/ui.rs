use std::char;
use std::cmp;
use std::io;
use std::str::FromStr;

use clap::Clap;
use sequence_trie::SequenceTrie;
use termion::{self, color, cursor, event, style};

use crate::error::ParseError;
use crate::{colors, model};

pub struct Ui<'a> {
    model: &'a mut model::Model<'a>,
    term_width: u16,
    line_offsets: Vec<usize>,
    matches: Vec<model::Match<'a>>,
    lookup_trie: SequenceTrie<char, usize>,
    focus_index: usize,
    focus_wrap_around: bool,
    rendering_colors: &'a UiColors,
    hint_alignment: &'a HintAlignment,
    hint_style: Option<HintStyle>,
}

impl<'a> Ui<'a> {
    pub fn new(
        model: &'a mut model::Model<'a>,
        unique_hint: bool,
        focus_wrap_around: bool,
        rendering_colors: &'a UiColors,
        hint_alignment: &'a HintAlignment,
        hint_style: Option<HintStyle>,
    ) -> Ui<'a> {
        let matches = model.matches(unique_hint);
        let lookup_trie = model::Model::build_lookup_trie(&matches);
        let focus_index = if model.reverse { matches.len() - 1 } else { 0 };

        let (term_width, _) = termion::terminal_size().unwrap_or((80u16, 30u16)); // .expect("Cannot read the terminal size.");
        let line_offsets = get_line_offsets(&model.lines, term_width);

        Ui {
            model,
            term_width,
            line_offsets,
            matches,
            lookup_trie,
            focus_index,
            focus_wrap_around,
            rendering_colors,
            hint_alignment,
            hint_style,
        }
    }

    /// Convert the `Match` text into the coordinates of the wrapped lines.
    ///
    /// Compute the new x offset of the text as the remainder of the line width
    /// (e.g. the match could start at offset 120 in a 80-width terminal, the new
    /// offset being 40).
    ///
    /// Compute the new y offset of the text as the initial y offset plus any
    /// additional offset due to previous split lines. This is obtained thanks to
    /// the `offset_per_line` member.
    pub fn map_coords_to_wrapped_space(&self, offset_x: usize, offset_y: usize) -> (usize, usize) {
        let line_width = self.term_width as usize;

        let new_offset_x = offset_x % line_width;
        let new_offset_y =
            self.line_offsets.get(offset_y as usize).unwrap() + offset_x / line_width;

        (new_offset_x, new_offset_y)
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
        } else if self.focus_index > 0 {
            self.focus_index -= 1;
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
        } else if self.focus_index < self.matches.len() - 1 {
            self.focus_index += 1;
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
    fn render_base_text(
        stdout: &mut dyn io::Write,
        lines: &[&str],
        line_offsets: &[usize],
        colors: &UiColors,
    ) {
        write!(
            stdout,
            "{bg_color}{fg_color}",
            fg_color = color::Fg(colors.text_fg.as_ref()),
            bg_color = color::Bg(colors.text_bg.as_ref()),
        )
        .unwrap();

        for (line_index, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim_end();

            if !trimmed_line.is_empty() {
                let offset_y: usize =
                    *(line_offsets.get(line_index)).expect("Cannot get offset_per_line.");

                write!(
                    stdout,
                    "{goto}{text}",
                    goto = cursor::Goto(1, offset_y as u16 + 1),
                    text = &trimmed_line,
                )
                .unwrap();
            }
        }

        write!(
            stdout,
            "{fg_reset}{bg_reset}",
            fg_reset = color::Fg(color::Reset),
            bg_reset = color::Bg(color::Reset),
        )
        .unwrap();
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
        colors: &UiColors,
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
        colors: &UiColors,
        hint_style: &Option<HintStyle>,
    ) {
        let fg_color = color::Fg(colors.hint_fg.as_ref());
        let bg_color = color::Bg(colors.hint_bg.as_ref());
        let fg_reset = color::Fg(color::Reset);
        let bg_reset = color::Bg(color::Reset);
        let goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1);

        match hint_style {
            None => {
                write!(
                    stdout,
                    "{goto}{bg_color}{fg_color}{hint}{fg_reset}{bg_reset}",
                    goto = goto,
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
                        goto = goto,
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
                        goto = goto,
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
                        goto = goto,
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
                        goto = goto,
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
        let (offset_x, offset_y) = self.map_coords_to_wrapped_space(offset_x, offset_y);

        Ui::render_matched_text(
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

            Ui::render_matched_hint(
                stdout,
                &mat.hint,
                (offset_x + extra_offset, offset_y),
                &self.rendering_colors,
                &self.hint_style,
            );
        }
    }

    /// Full nender the Ui on the provided writer.
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
    fn full_render(&self, stdout: &mut dyn io::Write) {
        // 1. Trim all lines and render non-empty ones.
        Ui::render_base_text(
            stdout,
            &self.model.lines,
            &self.line_offsets,
            &self.rendering_colors,
        );

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

        let mut uppercased = false;
        let mut typed_hint = String::new();

        self.full_render(writer);

        loop {
            // This is an option of a result of a key... Let's pop error cases first.
            let next_key = reader.keys().next();

            if next_key.is_none() {
                // Nothing in the buffer. Wait for a bit...
                std::thread::sleep(std::time::Duration::from_millis(25));
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
                event::Key::Char(_ch @ 'y') | event::Key::Char(_ch @ '\n') => {
                    let text = self.matches.get(self.focus_index).unwrap().text;
                    return Event::Match((text.to_string(), false));
                }
                event::Key::Char(_ch @ 'Y') => {
                    let text = self.matches.get(self.focus_index).unwrap().text;
                    return Event::Match((text.to_string(), true));
                }

                // Use a Trie or another data structure to determine
                // if the entered key belongs to a longer hint.
                // Attempts at finding a match with a corresponding hint.
                //
                // If any of the typed character is caps, the typed hint is
                // deemed as uppercased.
                event::Key::Char(ch) => {
                    let key = ch.to_string();
                    let lower_key = key.to_lowercase();

                    uppercased = uppercased || (key != lower_key);
                    typed_hint.push_str(&lower_key);

                    let node = self
                        .lookup_trie
                        .get_node(&typed_hint.chars().collect::<Vec<char>>());

                    if node.is_none() {
                        // An unknown key was entered.
                        return Event::Exit;
                    }

                    let node = node.unwrap();
                    if node.is_leaf() {
                        // The last key of a hint was entered.
                        let match_index = node.value().expect(
                            "By construction, the Lookup Trie should have a value for each leaf.",
                        );
                        let mat = self.matches.get(*match_index).expect("By construction, the value in a leaf should correspond to an existing hint.");
                        let text = mat.text.to_string();
                        return Event::Match((text, uppercased));
                    } else {
                        // The prefix of a hint was entered, but we
                        // still need more keys.
                        continue;
                    }
                }

                // Unknown keys are ignored.
                _ => (),
            }

            // End of event processing loop.
        }

        Event::Exit
    }

    /// Configure the terminal and display the `Ui`.
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

/// Compute each line's actual y offset if displayed in a terminal of width
/// `term_width`.
fn get_line_offsets(lines: &[&str], term_width: u16) -> Vec<usize> {
    lines
        .iter()
        .scan(0, |offset, &line| {
            // Save the value to return (yield is in unstable).
            let value = *offset;

            let line_width = line.trim_end().chars().count() as isize;

            // Amount of extra y space taken by this line.
            // If the line has n chars, on a term of width n, this does not
            // produce an extra line; it needs to exceed the width by 1 char.
            // In case the width is 0, we need to clamp line_width - 1 first.
            let extra = cmp::max(0, line_width - 1) as usize / term_width as usize;

            // Update the offset of the next line.
            *offset = *offset + 1 + extra;

            Some(value)
        })
        .collect()
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

/// Returned value after the `Ui` has finished listening to events.
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
        let line_offsets: Vec<usize> = (0..lines.len()).collect();

        let colors = UiColors {
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
        Ui::render_base_text(&mut writer, &lines, &line_offsets, &colors);

        let goto1 = cursor::Goto(1, 1);
        let goto2 = cursor::Goto(1, 2);
        let goto3 = cursor::Goto(1, 3);
        let goto6 = cursor::Goto(1, 6);
        assert_eq!(
            writer,
            format!(
                "{bg}{fg}{g1}some text{g2}* e006b06 - (12 days ago) swapper: Make quotes{g3}path: /usr/local/bin/git{g6}path: /usr/local/bin/cargo{fg_reset}{bg_reset}",
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
        let colors = UiColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        Ui::render_matched_text(&mut writer, text, focused, offset, &colors);

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
        let colors = UiColors {
            text_fg: Box::new(color::Black),
            text_bg: Box::new(color::White),
            focused_fg: Box::new(color::Red),
            focused_bg: Box::new(color::Blue),
            match_fg: Box::new(color::Green),
            match_bg: Box::new(color::Magenta),
            hint_fg: Box::new(color::Yellow),
            hint_bg: Box::new(color::Cyan),
        };

        Ui::render_matched_text(&mut writer, text, focused, offset, &colors);

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
        let colors = UiColors {
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

        Ui::render_matched_hint(
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
        let colors = UiColors {
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

        Ui::render_matched_hint(
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
        let colors = UiColors {
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

        Ui::render_matched_hint(
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

        let use_all_patterns = true;
        let named_pat = vec![];
        let custom_patterns = vec![];
        let alphabet = alphabets::Alphabet("abcd".to_string());
        let reverse = false;
        let mut model = model::Model::new(
            content,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom_patterns,
            reverse,
        );
        let term_width: u16 = 80;
        let line_offsets = get_line_offsets(&model.lines, term_width);
        let rendering_colors = UiColors {
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

        // create a Ui without any match
        let ui = Ui {
            model: &mut model,
            term_width,
            line_offsets,
            matches: vec![], // no matches
            lookup_trie: SequenceTrie::new(),
            focus_index: 0,
            focus_wrap_around: false,
            rendering_colors: &rendering_colors,
            hint_alignment: &hint_alignment,
            hint_style: None,
        };

        let mut writer = vec![];
        ui.full_render(&mut writer);

        let goto1 = cursor::Goto(1, 1);
        let goto3 = cursor::Goto(1, 3);

        let expected = format!(
            "{bg}{fg}{goto1}lorem 127.0.0.1 lorem\
        {goto3}Barcelona https://en.wikipedia.org/wiki/Barcelona -{fg_reset}{bg_reset}",
            goto1 = goto1,
            goto3 = goto3,
            fg = color::Fg(rendering_colors.text_fg.as_ref()),
            bg = color::Bg(rendering_colors.text_bg.as_ref()),
            fg_reset = color::Fg(color::Reset),
            bg_reset = color::Bg(color::Reset),
        );

        // println!("{:?}", writer);
        // println!("{:?}", expected.as_bytes());

        // println!("matches: {}", ui.matches.len());
        // println!("lines: {}", lines.len());

        assert_eq!(writer, expected.as_bytes());
    }

    #[test]
    /// Simulates rendering with matches.
    fn test_render_full_with_matches() {
        let content = "lorem 127.0.0.1 lorem

Barcelona https://en.wikipedia.org/wiki/Barcelona -   ";

        let use_all_patterns = true;
        let named_pat = vec![];
        let custom_patterns = vec![];
        let alphabet = alphabets::Alphabet("abcd".to_string());
        let reverse = true;
        let mut model = model::Model::new(
            content,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom_patterns,
            reverse,
        );
        let unique_hint = false;
        let wrap_around = false;

        let rendering_colors = UiColors {
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

        let ui = Ui::new(
            &mut model,
            unique_hint,
            wrap_around,
            &rendering_colors,
            &hint_alignment,
            hint_style,
        );

        let mut writer = vec![];
        ui.full_render(&mut writer);

        let expected_content = {
            let goto1 = cursor::Goto(1, 1);
            let goto3 = cursor::Goto(1, 3);

            format!(
                "{bg}{fg}{goto1}lorem 127.0.0.1 lorem\
        {goto3}Barcelona https://en.wikipedia.org/wiki/Barcelona -{fg_reset}{bg_reset}",
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

        assert_eq!(2, ui.matches.len());

        assert_eq!(writer, expected.as_bytes());
    }
}

// /// Holds color-related data, for clarity.
// ///
// /// - `focus_*` colors are used to render the currently focused matched text.
// /// - `normal_*` colors are used to render other matched text.
// /// - `hint_*` colors are used to render the hints.
#[derive(Clap, Debug)]
pub struct UiColors {
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
