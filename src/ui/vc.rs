use std::char;
use std::cmp;
use std::io;
use std::io::Write;

use termion::{self, color, cursor, event, screen::IntoAlternateScreen, style};
use unicode_width::UnicodeWidthChar;

use super::colors::UiColors;
use super::Selection;
use super::{HintAlignment, HintStyle};
use crate::{config::extended::OutputDestination, textbuf};

/// Describes where a line from the buffer is displayed on the screen and how
/// much vertical lines it takes.
///
/// The `pos_y` field is the actual vertical position due to wrapped lines
/// before this line. The `size` field is the number of screen lines occupied
/// by this line.
///
/// For example, given a buffer in which
///
/// - the first line is smaller than the screen width,
/// - the second line is slightly larger,
/// - and the third line is smaller than the screen width,
///
/// The corresponding `WrappedLine`s are
///
/// - the first `WrappedLine` has `pos_y: 0` and `size: 1`
/// - the second `WrappedLine` has `pos_y: 1` and `size: 2` (larger than screen
///   width)
/// - the third `WrappedLine` has `pos_y: 3` and `size: 1`
///
struct WrappedLine {
    pos_y: usize,
}

/// Manages the visible portion of content when total content exceeds terminal height.
///
/// The viewport tracks which rows of the wrapped content are currently visible
/// and provides methods for scrolling and coordinate translation.
struct Viewport {
    /// First visible row in wrapped content space (0-indexed).
    top_row: usize,
    /// Terminal height (number of visible rows).
    height: usize,
}

impl Viewport {
    /// Create a new viewport with the given terminal height.
    fn new(height: usize) -> Self {
        Viewport { top_row: 0, height }
    }

    /// Check if a content row is within the visible viewport.
    fn is_visible(&self, content_row: usize) -> bool {
        content_row >= self.top_row && content_row < self.top_row + self.height
    }

    /// Convert a content row to screen Y coordinate (1-indexed for termion).
    /// Returns None if the row is not visible.
    fn screen_y(&self, content_row: usize) -> Option<u16> {
        if self.is_visible(content_row) {
            Some((content_row - self.top_row + 1) as u16)
        } else {
            None
        }
    }

    /// Scroll the viewport to make a content row visible.
    /// Returns true if scrolling occurred.
    fn ensure_visible(&mut self, content_row: usize) -> bool {
        if content_row < self.top_row {
            // Row is above viewport, scroll up
            self.top_row = content_row;
            true
        } else if content_row >= self.top_row + self.height {
            // Row is below viewport, scroll down
            self.top_row = content_row - self.height + 1;
            true
        } else {
            false
        }
    }

    /// Scroll up by the given number of lines.
    /// Returns true if scrolling occurred.
    fn scroll_up(&mut self, lines: usize) -> bool {
        if self.top_row > 0 {
            self.top_row = self.top_row.saturating_sub(lines);
            true
        } else {
            false
        }
    }

    /// Scroll down by the given number of lines.
    /// Returns true if scrolling occurred.
    fn scroll_down(&mut self, lines: usize, max_content_height: usize) -> bool {
        let max_top = max_content_height.saturating_sub(self.height);
        if self.top_row < max_top {
            self.top_row = cmp::min(self.top_row + lines, max_top);
            true
        } else {
            false
        }
    }
}

pub struct ViewController<'a> {
    model: &'a textbuf::Model<'a>,
    term_width: u16,
    term_height: u16,
    wrapped_lines: Vec<WrappedLine>,
    total_content_height: usize,
    viewport: Viewport,
    focus_index: usize,
    focus_wrap_around: bool,
    default_output_destination: OutputDestination,
    rendering_colors: &'a UiColors,
    hint_alignment: &'a HintAlignment,
    hint_style: Option<HintStyle>,
}

impl<'a> ViewController<'a> {
    // Initialize {{{1

    pub fn new(
        model: &'a textbuf::Model<'a>,
        focus_wrap_around: bool,
        default_output_destination: OutputDestination,
        rendering_colors: &'a UiColors,
        hint_alignment: &'a HintAlignment,
        hint_style: Option<HintStyle>,
    ) -> ViewController<'a> {
        let focus_index = if model.reverse {
            model.spans.len().saturating_sub(1)
        } else {
            0
        };

        let (term_width, term_height) =
            termion::terminal_size().unwrap_or((80u16, 30u16));
        let wrapped_lines = compute_wrapped_lines(model.lines, term_width);

        // Compute total content height from wrapped lines
        let total_content_height = if wrapped_lines.is_empty() {
            0
        } else {
            // Last line's pos_y + 1 (for that line itself)
            // We need to account for the last line's wrapping too
            let last_idx = wrapped_lines.len() - 1;
            let last_line = model.lines.get(last_idx).unwrap_or(&"");
            let last_line_width = display_width(last_line.trim_end(), DEFAULT_TAB_WIDTH);
            let last_line_extra =
                cmp::max(0, last_line_width as isize - 1) as usize / term_width as usize;
            wrapped_lines[last_idx].pos_y + 1 + last_line_extra
        };

        let viewport = Viewport::new(term_height as usize);

        // Auto-scroll to the initially focused span
        let mut vc = ViewController {
            model,
            term_width,
            term_height,
            wrapped_lines,
            total_content_height,
            viewport,
            focus_index,
            focus_wrap_around,
            default_output_destination,
            rendering_colors,
            hint_alignment,
            hint_style,
        };

        // Scroll to make the initially focused span visible
        if !model.spans.is_empty() {
            let content_row = vc.span_content_row(&model.spans[focus_index]);
            vc.viewport.ensure_visible(content_row);
        }

        vc
    }

    // }}}
    // Coordinates {{{1

    /// Returns the adjusted position of a given `Span` within the buffer
    /// line.
    ///
    /// This adjustment is necessary because the span's x coordinate is a byte
    /// offset, but we need the display column. The display column accounts for:
    /// - Tab expansion to tab stops (multiples of 8 columns)
    /// - Unicode character widths (combining chars, wide CJK chars, etc.)
    ///
    /// This computation must happen before mapping the span position to the
    /// wrapped screen space.
    fn adjusted_span_position(&self, span: &textbuf::Span<'a>) -> (usize, usize) {
        let pos_x = {
            let line = &self.model.lines[span.y as usize];
            let prefix = &line[0..span.x as usize];
            display_width(prefix, DEFAULT_TAB_WIDTH)
        };
        let pos_y = span.y as usize;

        (pos_x, pos_y)
    }

    /// Convert the `Span` text into the coordinates of the wrapped lines.
    ///
    /// Compute the new x position of the text as the remainder of the line width
    /// (e.g. the Span could start at position 120 in a 80-width terminal, the new
    /// position being 40).
    ///
    /// Compute the new y position of the text as the initial y position plus any
    /// additional offset due to previous split lines. This is obtained thanks to
    /// the `wrapped_lines` field.
    fn map_coords_to_wrapped_space(&self, pos_x: usize, pos_y: usize) -> (usize, usize) {
        let line_width = self.term_width as usize;

        let new_pos_x = pos_x % line_width;
        let new_pos_y = self.wrapped_lines[pos_y].pos_y + pos_x / line_width;

        (new_pos_x, new_pos_y)
    }

    /// Returns the content row (in wrapped space) where a span is located.
    fn span_content_row(&self, span: &textbuf::Span<'a>) -> usize {
        let (pos_x, pos_y) = self.adjusted_span_position(span);
        let (_, content_row) = self.map_coords_to_wrapped_space(pos_x, pos_y);
        content_row
    }

    // }}}
    // Focus management {{{1

    /// Move focus onto the previous hint, returning both the index of the
    /// previously focused Span, and the index of the newly focused one.
    fn prev_focus_index(&mut self) -> (usize, usize) {
        let old_index = self.focus_index;
        if self.focus_wrap_around {
            if self.focus_index == 0 {
                self.focus_index = self.model.spans.len() - 1;
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
    /// previously focused Span, and the index of the newly focused one.
    fn next_focus_index(&mut self) -> (usize, usize) {
        let old_index = self.focus_index;
        if self.focus_wrap_around {
            if self.focus_index == self.model.spans.len() - 1 {
                self.focus_index = 0;
            } else {
                self.focus_index += 1;
            }
        } else if self.focus_index < self.model.spans.len() - 1 {
            self.focus_index += 1;
        }
        let new_index = self.focus_index;
        (old_index, new_index)
    }

    /// Handle focus change with scrolling. Returns true if a full render is needed.
    fn handle_focus_change(
        &mut self,
        old_index: usize,
        new_index: usize,
        writer: &mut dyn io::Write,
    ) {
        // Check if the new focused span is visible
        let content_row = self.span_content_row(&self.model.spans[new_index]);
        let scrolled = self.viewport.ensure_visible(content_row);

        if scrolled {
            // Viewport changed, need full render
            self.full_render(writer);
        } else {
            // Viewport didn't change, efficient diff render
            self.diff_render(writer, old_index, new_index);
        }
    }

    // }}}
    // Rendering {{{1

    /// Render entire model lines on provided writer, respecting viewport bounds.
    ///
    /// This renders the basic content on which spans and hints can be rendered.
    ///
    /// # Notes
    /// - All trailing whitespaces are trimmed, empty lines are skipped.
    /// - Only content within the viewport is rendered.
    /// - Long lines that wrap are handled correctly across viewport boundaries.
    /// - This writes directly on the writer, avoiding extra allocation.
    fn render_base_text(
        stdout: &mut dyn io::Write,
        lines: &[&str],
        wrapped_lines: &[WrappedLine],
        colors: &UiColors,
        viewport: &Viewport,
        term_width: u16,
    ) {
        write!(
            stdout,
            "{bg_color}{fg_color}",
            fg_color = color::Fg(colors.text_fg),
            bg_color = color::Bg(colors.text_bg),
        )
        .unwrap();

        let line_width = term_width as usize;

        for (line_index, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim_end();

            if trimmed_line.is_empty() {
                continue;
            }

            let base_content_row = wrapped_lines[line_index].pos_y;
            let line_display_width = display_width(trimmed_line, DEFAULT_TAB_WIDTH);
            let num_wrapped_rows = if line_display_width == 0 {
                1
            } else {
                line_display_width.div_ceil(line_width)
            };

            // Check if any part of this line is visible
            let line_end_row = base_content_row + num_wrapped_rows - 1;
            if line_end_row < viewport.top_row
                || base_content_row >= viewport.top_row + viewport.height
            {
                continue; // Line is entirely outside viewport
            }

            // For each sub-row of this potentially wrapped line
            for sub_row in 0..num_wrapped_rows {
                let content_row = base_content_row + sub_row;
                if let Some(screen_y) = viewport.screen_y(content_row) {
                    // Calculate which portion of the line to render for this sub-row
                    // We need to find the character offset for this sub-row
                    let start_col = sub_row * line_width;
                    let end_col = start_col + line_width;

                    // Extract the substring for this sub-row by iterating through display widths
                    let mut col = 0;
                    let mut char_start: Option<usize> = None;
                    let mut char_end = trimmed_line.len();

                    for (byte_idx, ch) in trimmed_line.char_indices() {
                        // Set char_start when we reach or pass start_col
                        if char_start.is_none() && col >= start_col {
                            char_start = Some(byte_idx);
                        }

                        let ch_width = if ch == '\t' {
                            ((col / DEFAULT_TAB_WIDTH) + 1) * DEFAULT_TAB_WIDTH - col
                        } else {
                            ch.width().unwrap_or(0)
                        };
                        col += ch_width;

                        // Set char_end when we pass end_col
                        if col > end_col {
                            char_end = byte_idx;
                            break;
                        }
                    }

                    let char_start = char_start.unwrap_or(0);
                    let sub_text = &trimmed_line[char_start..char_end];
                    if !sub_text.is_empty() {
                        write!(
                            stdout,
                            "{goto}{text}",
                            goto = cursor::Goto(1, screen_y),
                            text = sub_text,
                        )
                        .unwrap();
                    }
                }
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

    /// Render the Span's `text` field on provided writer using the `span_*g` color.
    ///
    /// If a Mach is "focused", it is then rendered with the `focused_*g` colors.
    ///
    /// # Arguments
    /// - `pos` is (x_in_content_space, screen_y) where screen_y is 1-indexed
    ///
    /// # Note
    ///
    /// This writes directly on the writer, avoiding extra allocation.
    fn render_span_text(
        stdout: &mut dyn io::Write,
        text: &str,
        focused: bool,
        pos: (usize, u16),
        colors: &UiColors,
    ) {
        // To help identify it, the span thas has focus is rendered with a dedicated color.
        let (fg_color, bg_color) = if focused {
            (&colors.focused_fg, &colors.focused_bg)
        } else {
            (&colors.span_fg, &colors.span_bg)
        };

        // Render just the Span's text on top of existing content.
        write!(
            stdout,
            "{goto}{bg_color}{fg_color}{text}{fg_reset}{bg_reset}",
            goto = cursor::Goto(pos.0 as u16 + 1, pos.1),
            fg_color = color::Fg(*fg_color),
            bg_color = color::Bg(*bg_color),
            fg_reset = color::Fg(color::Reset),
            bg_reset = color::Bg(color::Reset),
            text = &text,
        )
        .unwrap();
    }

    /// Render a Span's `hint` field on the provided writer.
    ///
    /// This renders the hint according to some provided style:
    /// - just colors
    /// - styled (bold, italic, underlined) with colors
    /// - surrounding the hint's text with some delimiters, see
    ///   `HintStyle::Delimited`.
    ///
    /// # Arguments
    /// - `pos` is (x_in_content_space, screen_y) where screen_y is 1-indexed
    ///
    /// # Note
    ///
    /// This writes directly on the writer, avoiding extra allocation.
    fn render_span_hint(
        stdout: &mut dyn io::Write,
        hint_text: &str,
        pos: (usize, u16),
        colors: &UiColors,
        hint_style: &Option<HintStyle>,
    ) {
        let fg_color = color::Fg(colors.hint_fg);
        let bg_color = color::Bg(colors.hint_bg);
        let fg_reset = color::Fg(color::Reset);
        let bg_reset = color::Bg(color::Reset);
        let goto = cursor::Goto(pos.0 as u16 + 1, pos.1);

        match hint_style {
            None => {
                write!(
                    stdout,
                    "{goto}{bg_color}{fg_color}{hint_text}{fg_reset}{bg_reset}",
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
                        sty_reset = style::Reset, // NoBold is not sufficient
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
                        "{goto}{bg_color}{fg_color}{opening}{hint_text}{closing}{fg_reset}{bg_reset}",
                    )
                    .unwrap();
                }
            },
        }
    }

    /// Convenience function that renders both the text span and its hint,
    /// if focused. Only renders if the span is visible in the viewport.
    fn render_span(&self, stdout: &mut dyn io::Write, span: &textbuf::Span<'a>, focused: bool) {
        let text = span.text;

        let (pos_x, pos_y) = self.adjusted_span_position(span);
        let (pos_x, content_row) = self.map_coords_to_wrapped_space(pos_x, pos_y);

        // Check if span is visible in viewport
        let screen_y = match self.viewport.screen_y(content_row) {
            Some(y) => y,
            None => return, // Span not visible, skip rendering
        };

        ViewController::render_span_text(
            stdout,
            text,
            focused,
            (pos_x, screen_y),
            self.rendering_colors,
        );

        if !focused {
            // If not focused, render the hint (e.g. "eo") as an overlay on
            // top of the rendered text span, aligned at its leading or the
            // trailing edge.
            let offset = match self.hint_alignment {
                HintAlignment::Leading => 0,
                HintAlignment::Trailing => text.len() - span.hint.len(),
            };

            ViewController::render_span_hint(
                stdout,
                &span.hint,
                (pos_x + offset, screen_y),
                self.rendering_colors,
                &self.hint_style,
            );
        }
    }

    /// Render scroll position indicator in bottom-right corner.
    /// Only shown when content exceeds viewport height.
    fn render_scroll_indicator(&self, stdout: &mut dyn io::Write) {
        if self.total_content_height <= self.viewport.height {
            return; // No scrolling needed, no indicator
        }

        let max_top = self.total_content_height.saturating_sub(self.viewport.height);
        let indicator = format!("[{}/{}]", self.viewport.top_row + 1, max_top + 1);

        // Render in bottom-right corner with dim styling
        let x_pos = self.term_width.saturating_sub(indicator.len() as u16);
        write!(
            stdout,
            "{goto}{dim}{text}{reset}",
            goto = cursor::Goto(x_pos, self.term_height),
            dim = style::Faint,
            text = indicator,
            reset = style::Reset,
        )
        .unwrap();
    }

    /// Full render the Ui on the provided writer.
    ///
    /// This renders in 3 phases:
    /// - all lines are rendered verbatim (within viewport)
    /// - each Span's `text` is rendered as an overlay on top of it
    /// - each Span's `hint` text is rendered as a final overlay
    ///
    /// Depending on the value of `self.hint_alignment`, the hint can be
    /// rendered on the leading edge of the underlying Span's `text`, or on
    /// the trailing edge.
    ///
    /// # Note
    ///
    /// Multibyte characters are taken into account, so that the Span's `text`
    /// and `hint` are rendered in their proper position.
    fn full_render(&self, stdout: &mut dyn io::Write) {
        // Clear screen before rendering
        write!(stdout, "{}", termion::clear::All).unwrap();

        // 1. Trim all lines and render non-empty ones within viewport.
        ViewController::render_base_text(
            stdout,
            self.model.lines,
            &self.wrapped_lines,
            self.rendering_colors,
            &self.viewport,
            self.term_width,
        );

        // 2. Render spans (only visible ones)
        for (index, span) in self.model.spans.iter().enumerate() {
            let focused = index == self.focus_index;
            self.render_span(stdout, span, focused);
        }

        // 3. Render scroll indicator if content exceeds viewport
        self.render_scroll_indicator(stdout);

        stdout.flush().unwrap();
    }

    /// Render the previous span with its hint, and render the newly focused
    /// span without its hint. This is more efficient than a full render.
    fn diff_render(
        &self,
        stdout: &mut dyn io::Write,
        old_focus_index: usize,
        new_focus_index: usize,
    ) {
        // Render the previously focused span as non-focused
        let span = self.model.spans.get(old_focus_index).unwrap();
        let focused = false;
        self.render_span(stdout, span, focused);

        // Render the previously focused span as non-focused
        let span = self.model.spans.get(new_focus_index).unwrap();
        let focused = true;
        self.render_span(stdout, span, focused);

        stdout.flush().unwrap();
    }

    // }}}
    // Listening {{{1

    /// Listen to keys entered on stdin, moving focus accordingly, or
    /// selecting one span.
    ///
    /// # Panics
    ///
    /// - This function panics if termion cannot read the entered keys on stdin.
    fn listen(&mut self, reader: &mut dyn io::Read, writer: &mut dyn io::Write) -> Event {
        use termion::input::TermRead; // Trait for `reader.keys().next()`.

        if self.model.spans.is_empty() {
            return Event::Exit;
        }

        let mut typed_hint = String::new();
        let mut uppercased = false;
        let mut output_destination = self.default_output_destination.clone();

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
                panic!("{}", err);
            }

            match key_res.unwrap() {
                event::Key::Esc => {
                    break;
                }

                // Move focus to next/prev span with viewport scrolling.
                event::Key::Up => {
                    let (old_index, focused_index) = self.prev_focus_index();
                    self.handle_focus_change(old_index, focused_index, writer);
                }
                event::Key::Down => {
                    let (old_index, focused_index) = self.next_focus_index();
                    self.handle_focus_change(old_index, focused_index, writer);
                }
                event::Key::Left => {
                    let (old_index, focused_index) = self.prev_focus_index();
                    self.handle_focus_change(old_index, focused_index, writer);
                }
                event::Key::Right => {
                    let (old_index, focused_index) = self.next_focus_index();
                    self.handle_focus_change(old_index, focused_index, writer);
                }
                event::Key::Char(_ch @ 'n') => {
                    let (old_index, focused_index) = if self.model.reverse {
                        self.prev_focus_index()
                    } else {
                        self.next_focus_index()
                    };
                    self.handle_focus_change(old_index, focused_index, writer);
                }
                event::Key::Char(_ch @ 'N') => {
                    let (old_index, focused_index) = if self.model.reverse {
                        self.next_focus_index()
                    } else {
                        self.prev_focus_index()
                    };
                    self.handle_focus_change(old_index, focused_index, writer);
                }

                // Manual scrolling with PageUp/PageDown
                event::Key::PageUp => {
                    let scroll_amount = self.viewport.height / 2;
                    if self.viewport.scroll_up(scroll_amount) {
                        self.full_render(writer);
                    }
                }
                event::Key::PageDown => {
                    let scroll_amount = self.viewport.height / 2;
                    if self.viewport.scroll_down(scroll_amount, self.total_content_height) {
                        self.full_render(writer);
                    }
                }

                // Yank/copy
                event::Key::Char(_ch @ 'y') | event::Key::Char(_ch @ '\n') => {
                    let text = self.model.spans.get(self.focus_index).unwrap().text;
                    return Event::Select(Selection {
                        text: text.to_string(),
                        uppercased: false,
                        output_destination,
                    });
                }
                event::Key::Char(_ch @ 'Y') => {
                    let text = self.model.spans.get(self.focus_index).unwrap().text;
                    return Event::Select(Selection {
                        text: text.to_string(),
                        uppercased: true,
                        output_destination,
                    });
                }

                event::Key::Char(_ch @ ' ') => {
                    output_destination.toggle();
                    let message = format!("output destination: `{output_destination}`");
                    duct::cmd!("tmux", "display-message", &message)
                        .run()
                        .expect("could not make tmux display the message.");
                    continue;
                }

                // Use a Trie or another data structure to determine
                // if the entered key belongs to a longer hint.
                // Attempts at finding a span with a corresponding hint.
                //
                // If any of the typed character is caps, the typed hint is
                // deemed as uppercased.
                event::Key::Char(ch) => {
                    let key = ch.to_string();
                    let lower_key = key.to_lowercase();

                    uppercased = uppercased || (key != lower_key);
                    typed_hint.push_str(&lower_key);

                    let node = self
                        .model
                        .lookup_trie
                        .get_node(&typed_hint.chars().collect::<Vec<char>>());

                    if node.is_none() {
                        // A key outside the alphabet was entered.
                        return Event::Exit;
                    }

                    let node = node.unwrap();
                    if node.is_leaf() {
                        // The last key of a hint was entered.
                        let span_index = node.value().expect(
                            "By construction, the Lookup Trie should have a value for each leaf.",
                        );
                        let span = self.model.spans.get(*span_index).expect("By construction, the value in a leaf should correspond to an existing hint.");
                        let text = span.text.to_string();
                        return Event::Select(Selection {
                            text,
                            uppercased,
                            output_destination,
                        });
                    }
                    // The prefix of a hint was entered, but we
                    // still need more keys.
                }

                // Unknown keys are ignored.
                _ => (),
            }

            // End of event processing loop.
        }

        Event::Exit
    }

    // }}}
    // Presenting {{{1

    /// Configure the terminal and display the `Ui`.
    ///
    /// - Setup steps: switch to alternate screen, switch to raw mode, hide the cursor.
    /// - Teardown steps: show cursor, back to main screen.
    pub fn present(&mut self) -> Option<Selection> {
        use termion::raw::IntoRawMode;

        let mut stdin = termion::async_stdin();
        let mut stdout = io::stdout()
            .into_raw_mode()
            .expect("Cannot access alternate screen.")
            .into_alternate_screen()
            .expect("Cannot access alternate screen.");

        // stdout.write(cursor::Hide.into()).unwrap();
        write!(stdout, "{}", cursor::Hide).unwrap();

        let selection = match self.listen(&mut stdin, &mut stdout) {
            Event::Exit => None,
            Event::Select(selection) => Some(selection),
        };

        write!(stdout, "{}", cursor::Show).unwrap();

        selection
    }

    // }}}
}

/// Default tab width in terminal columns.
const DEFAULT_TAB_WIDTH: usize = 8;

/// Compute the display width of a string, accounting for tab expansion and Unicode
/// character widths.
///
/// Tabs expand to the next tab stop (multiples of `tab_width`). Unicode characters
/// use their proper display width (e.g., CJK characters are 2 columns, combining
/// characters are 0 columns).
fn display_width(s: &str, tab_width: usize) -> usize {
    let mut col = 0;
    for ch in s.chars() {
        if ch == '\t' {
            // Expand tab to next tab stop
            col = ((col / tab_width) + 1) * tab_width;
        } else {
            col += ch.width().unwrap_or(0);
        }
    }
    col
}

/// Compute each line's actual y position and size if displayed in a terminal of width
/// `term_width`.
fn compute_wrapped_lines(lines: &[&str], term_width: u16) -> Vec<WrappedLine> {
    lines
        .iter()
        .scan(0, |position, &line| {
            // Save the value to return (yield is in unstable).
            let value = *position;

            let line_width = display_width(line.trim_end(), DEFAULT_TAB_WIDTH) as isize;

            // Amount of extra y space taken by this line.
            // If the line has n chars, on a term of width n, this does not
            // produce an extra line; it needs to exceed the width by 1 char.
            // In case the width is 0, we need to first clamp line_width - 1.
            let extra = cmp::max(0, line_width - 1) as usize / term_width as usize;

            // Update the position of the next line.
            *position += 1 + extra;

            Some(WrappedLine {
                pos_y: value,
                // size: 1 + extra,
            })
        })
        .collect()
}

/// Returned value after the `Ui` has finished listening to events.
enum Event {
    /// Exit with no selected spans,
    Exit,
    /// The selected span of text and whether it was selected with uppercase.
    Select(Selection),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{textbuf::alphabet, ui::colors};

    #[test]
    fn test_render_all_lines() {
        let content = "some text
* e006b06 - (12 days ago) swapper: Make quotes
path: /usr/local/bin/git


path: /usr/local/bin/cargo";
        let lines: Vec<&str> = content.split('\n').collect();
        let wrapped_lines: Vec<WrappedLine> = vec![
            WrappedLine { pos_y: 0 },
            WrappedLine { pos_y: 1 },
            WrappedLine { pos_y: 2 },
            WrappedLine { pos_y: 3 },
            WrappedLine { pos_y: 4 },
            WrappedLine { pos_y: 5 },
        ];

        let colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };

        // Viewport large enough to see all content
        let viewport = Viewport::new(10);
        let term_width = 80u16;

        let mut writer = vec![];
        ViewController::render_base_text(
            &mut writer,
            &lines,
            &wrapped_lines,
            &colors,
            &viewport,
            term_width,
        );

        let goto1 = cursor::Goto(1, 1);
        let goto2 = cursor::Goto(1, 2);
        let goto3 = cursor::Goto(1, 3);
        let goto6 = cursor::Goto(1, 6);
        assert_eq!(
            writer,
            format!(
                "{bg}{fg}{g1}some text{g2}* e006b06 - (12 days ago) swapper: Make quotes{g3}path: /usr/local/bin/git{g6}path: /usr/local/bin/cargo{fg_reset}{bg_reset}",
                g1 = goto1, g2 = goto2, g3 = goto3, g6 = goto6,
                fg = color::Fg(colors.text_fg),
                bg = color::Bg(colors.text_bg),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                )
            .as_bytes()
            );
    }

    #[test]
    fn test_render_focused_span_text() {
        let mut writer = vec![];
        let text = "https://en.wikipedia.org/wiki/Barcelona";
        let focused = true;
        // Position is (x_in_content_space, screen_y) where screen_y is 1-indexed
        let position: (usize, u16) = (3, 2);
        let colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };

        ViewController::render_span_text(&mut writer, text, focused, position, &colors);

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{text}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.focused_fg),
                bg = color::Bg(colors.focused_bg),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                text = &text,
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_render_span_text() {
        let mut writer = vec![];
        let text = "https://en.wikipedia.org/wiki/Barcelona";
        let focused = false;
        // Position is (x_in_content_space, screen_y) where screen_y is 1-indexed
        let position: (usize, u16) = (3, 2);
        let colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };

        ViewController::render_span_text(&mut writer, text, focused, position, &colors);

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{text}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.span_fg),
                bg = color::Bg(colors.span_bg),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                text = &text,
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_render_unstyled_span_hint() {
        let mut writer = vec![];
        let hint_text = "eo";
        // Position is (x_in_content_space, screen_y) where screen_y is 1-indexed
        let position: (usize, u16) = (3, 2);
        let colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };

        let offset = 0;
        let hint_style = None;

        ViewController::render_span_hint(
            &mut writer,
            hint_text,
            (position.0 + offset, position.1),
            &colors,
            &hint_style,
        );

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{text}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.hint_fg),
                bg = color::Bg(colors.hint_bg),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset),
                text = "eo",
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_render_underlined_span_hint() {
        let mut writer = vec![];
        let hint_text = "eo";
        // Position is (x_in_content_space, screen_y) where screen_y is 1-indexed
        let position: (usize, u16) = (3, 2);
        let colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };

        let offset = 0;
        let hint_style = Some(HintStyle::Underline);

        ViewController::render_span_hint(
            &mut writer,
            hint_text,
            (position.0 + offset, position.1),
            &colors,
            &hint_style,
        );

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{sty}{text}{sty_reset}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.hint_fg),
                bg = color::Bg(colors.hint_bg),
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
    fn test_render_bracketed_span_hint() {
        let mut writer = vec![];
        let hint_text = "eo";
        // Position is (x_in_content_space, screen_y) where screen_y is 1-indexed
        let position: (usize, u16) = (3, 2);
        let colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };

        let offset = 0;
        let hint_style = Some(HintStyle::Surround('{', '}'));

        ViewController::render_span_hint(
            &mut writer,
            hint_text,
            (position.0 + offset, position.1),
            &colors,
            &hint_style,
        );

        assert_eq!(
            writer,
            format!(
                "{goto}{bg}{fg}{bra}{text}{bra_close}{fg_reset}{bg_reset}",
                goto = cursor::Goto(4, 2),
                fg = color::Fg(colors.hint_fg),
                bg = color::Bg(colors.hint_bg),
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
    /// Simulates rendering without any span.
    fn test_render_full_without_available_spans() {
        let buffer = "lorem 127.0.0.1 lorem

Barcelona https://en.wikipedia.org/wiki/Barcelona -   ";
        let lines = buffer.split('\n').collect::<Vec<_>>();

        let use_all_patterns = false;
        let named_pat = vec![];
        let custom_patterns = vec![];
        let alphabet = alphabet::Alphabet("abcd".to_string());
        let reverse = false;
        let unique_hint = false;
        let mut model = textbuf::Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom_patterns,
            reverse,
            unique_hint,
        );
        let term_width: u16 = 80;
        let term_height: u16 = 30;
        let wrapped_lines = compute_wrapped_lines(model.lines, term_width);

        // Compute total content height
        let total_content_height = if wrapped_lines.is_empty() {
            0
        } else {
            let last_idx = wrapped_lines.len() - 1;
            wrapped_lines[last_idx].pos_y + 1
        };

        let rendering_colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };
        let hint_alignment = HintAlignment::Leading;
        let viewport = Viewport::new(term_height as usize);

        // create a Ui without any span
        let ui = ViewController {
            model: &mut model,
            term_width,
            term_height,
            wrapped_lines,
            total_content_height,
            viewport,
            focus_index: 0,
            focus_wrap_around: false,
            default_output_destination: OutputDestination::Tmux,
            rendering_colors: &rendering_colors,
            hint_alignment: &hint_alignment,
            hint_style: None,
        };

        let mut writer = vec![];
        ui.full_render(&mut writer);

        let goto1 = cursor::Goto(1, 1);
        let goto3 = cursor::Goto(1, 3);

        let expected = format!(
            "{clear}{bg}{fg}{goto1}lorem 127.0.0.1 lorem\
        {goto3}Barcelona https://en.wikipedia.org/wiki/Barcelona -{fg_reset}{bg_reset}",
            clear = termion::clear::All,
            goto1 = goto1,
            goto3 = goto3,
            fg = color::Fg(rendering_colors.text_fg),
            bg = color::Bg(rendering_colors.text_bg),
            fg_reset = color::Fg(color::Reset),
            bg_reset = color::Bg(color::Reset),
        );

        // println!("{:?}", writer);
        // println!("{:?}", expected.as_bytes());

        assert_eq!(writer, expected.as_bytes());
    }

    #[test]
    /// Simulates rendering with available spans.
    fn test_render_full_with_spans() {
        let buffer = "lorem 127.0.0.1 lorem

Barcelona https://en.wikipedia.org/wiki/Barcelona -   ";
        let lines = buffer.split('\n').collect::<Vec<_>>();

        let use_all_patterns = true;
        let named_pat = vec![];
        let custom_patterns = vec![];
        let alphabet = alphabet::Alphabet("abcd".to_string());
        let reverse = true;
        let unique_hint = false;
        let model = textbuf::Model::new(
            &lines,
            &alphabet,
            use_all_patterns,
            &named_pat,
            &custom_patterns,
            reverse,
            unique_hint,
        );
        let wrap_around = false;
        let default_output_destination = OutputDestination::Tmux;

        let rendering_colors = UiColors {
            text_fg: colors::BLACK,
            text_bg: colors::WHITE,
            focused_fg: colors::RED,
            focused_bg: colors::BLUE,
            span_fg: colors::GREEN,
            span_bg: colors::MAGENTA,
            hint_fg: colors::YELLOW,
            hint_bg: colors::CYAN,
        };
        let hint_alignment = HintAlignment::Leading;
        let hint_style = None;

        let ui = ViewController::new(
            &model,
            wrap_around,
            default_output_destination,
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
                "{clear}{bg}{fg}{goto1}lorem 127.0.0.1 lorem\
        {goto3}Barcelona https://en.wikipedia.org/wiki/Barcelona -{fg_reset}{bg_reset}",
                clear = termion::clear::All,
                goto1 = goto1,
                goto3 = goto3,
                fg = color::Fg(rendering_colors.text_fg),
                bg = color::Bg(rendering_colors.text_bg),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset)
            )
        };

        let expected_span1_text = {
            let goto7_1 = cursor::Goto(7, 1);
            format!(
                "{goto7_1}{span_bg}{span_fg}127.0.0.1{fg_reset}{bg_reset}",
                goto7_1 = goto7_1,
                span_fg = color::Fg(rendering_colors.span_fg),
                span_bg = color::Bg(rendering_colors.span_bg),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset)
            )
        };

        let expected_span1_hint = {
            let goto7_1 = cursor::Goto(7, 1);

            format!(
                "{goto7_1}{hint_bg}{hint_fg}b{fg_reset}{bg_reset}",
                goto7_1 = goto7_1,
                hint_fg = color::Fg(rendering_colors.hint_fg),
                hint_bg = color::Bg(rendering_colors.hint_bg),
                fg_reset = color::Fg(color::Reset),
                bg_reset = color::Bg(color::Reset)
            )
        };

        let expected_span2_text = {
            let goto11_3 = cursor::Goto(11, 3);
            format!(
        "{goto11_3}{focus_bg}{focus_fg}https://en.wikipedia.org/wiki/Barcelona{fg_reset}{bg_reset}",
        goto11_3 = goto11_3,
        focus_fg = color::Fg(rendering_colors.focused_fg),
        focus_bg = color::Bg(rendering_colors.focused_bg),
        fg_reset = color::Fg(color::Reset),
        bg_reset = color::Bg(color::Reset)
      )
        };

        // Because reverse is true, this second span is focused,
        // then the hint should not be rendered.

        // let expected_span2_hint = {
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
            expected_span1_text,
            expected_span1_hint,
            expected_span2_text,
            // expected_span2_hint,
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

        assert_eq!(2, ui.model.spans.len());

        assert_eq!(writer, expected.as_bytes());
    }

    #[test]
    fn test_display_width_without_tabs() {
        // Regular ASCII text
        assert_eq!(display_width("hello", 8), 5);
        assert_eq!(display_width("", 8), 0);
        assert_eq!(display_width("abc", 8), 3);
    }

    #[test]
    fn test_display_width_with_tabs() {
        // Tab at position 0 expands to 8 columns
        assert_eq!(display_width("\t", 8), 8);
        // Tab at position 1 expands to column 8 (7 more columns)
        assert_eq!(display_width("a\t", 8), 8);
        // Tab at position 7 expands to column 8 (1 more column)
        assert_eq!(display_width("1234567\t", 8), 8);
        // Tab at position 8 expands to column 16
        assert_eq!(display_width("12345678\t", 8), 16);
        // Multiple tabs
        assert_eq!(display_width("\t\t", 8), 16);
        // Tab followed by text
        assert_eq!(display_width("\tfile.txt", 8), 16); // 8 + 8
    }

    #[test]
    fn test_display_width_git_status_style() {
        // Simulates git status output with leading tab
        assert_eq!(display_width("\tTODO.md", 8), 15); // 8 (tab) + 7 (TODO.md)
        assert_eq!(display_width("\tsrc/textbuf/toto.rs", 8), 27); // 8 + 19
    }
}
