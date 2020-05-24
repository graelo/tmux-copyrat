use super::{colors, state};
use clap::Clap;
use std::char;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{color, cursor, style};

pub struct View<'a> {
  state: &'a mut state::State<'a>,
  matches: Vec<state::Match<'a>>,
  focus_index: usize,
  multi: bool,
  hint_alignment: HintAlignment,
  rendering_colors: &'a ViewColors,
  hint_style: Option<HintStyle>,
}

/// Holds color-related data, for clarity.
///
/// - `focus_*` colors are used to render the currently focused matched text.
/// - `normal_*` colors are used to render other matched text.
/// - `hint_*` colors are used to render the hints.
#[derive(Clap, Debug)]
pub struct ViewColors {
  /// Foreground color for matches.
  #[clap(long, default_value = "green",
                parse(try_from_str = colors::parse_color))]
  match_fg: Box<dyn color::Color>,

  /// Background color for matches.
  #[clap(long, default_value = "black",
                parse(try_from_str = colors::parse_color))]
  match_bg: Box<dyn color::Color>,

  /// Foreground color for the focused match.
  #[clap(long, default_value = "blue",
                parse(try_from_str = colors::parse_color))]
  focused_fg: Box<dyn color::Color>,

  /// Background color for the focused match.
  #[clap(long, default_value = "black",
                parse(try_from_str = colors::parse_color))]
  focused_bg: Box<dyn color::Color>,

  /// Foreground color for hints.
  #[clap(long, default_value = "white",
                parse(try_from_str = colors::parse_color))]
  hint_fg: Box<dyn color::Color>,

  /// Background color for hints.
  #[clap(long, default_value = "black",
                parse(try_from_str = colors::parse_color))]
  hint_bg: Box<dyn color::Color>,
}

/// Describes if, during rendering, a hint should aligned to the leading edge of
/// the matched text, or to its trailing edge.
#[derive(Debug, Clap)]
pub enum HintAlignment {
  Leading,
  Trailing,
}

/// Describes the style of contrast to be used during rendering of the hint's
/// text.
///
/// # Note
/// In practice, this is wrapped in an `Option`, so that the hint's text can be rendered with no style.
pub enum HintStyle {
  /// The hint's text will be underlined (leveraging `termion::style::Underline`).
  Underlined,
  /// The hint's text will be surrounded by these chars.
  Surrounded(char, char),
}

/// Returned value after the `View` has finished listening to events.
enum CaptureEvent {
  /// Exit with no selected matches,
  Exit,
  /// A vector of matched text and whether it was selected with uppercase.
  Hint(Vec<(String, bool)>),
}

impl<'a> View<'a> {
  pub fn new(
    state: &'a mut state::State<'a>,
    multi: bool,
    reversed: bool,
    unique: bool,
    hint_alignment: HintAlignment,
    rendering_colors: &'a ViewColors,
    hint_style: Option<HintStyle>,
  ) -> View<'a> {
    let matches = state.matches(reversed, unique);
    let focus_index = if reversed { matches.len() - 1 } else { 0 };

    View {
      state,
      matches,
      focus_index,
      multi,
      hint_alignment,
      rendering_colors,
      hint_style,
    }
  }

  /// Move focus onto the previous hint.
  pub fn prev(&mut self) {
    if self.focus_index > 0 {
      self.focus_index -= 1;
    }
  }

  /// Move focus onto the next hint.
  pub fn next(&mut self) {
    if self.focus_index < self.matches.len() - 1 {
      self.focus_index += 1;
    }
  }

  /// Render entire state lines on provided writer.
  ///
  /// This renders the basic content on which matches and hints can be rendered.
  ///
  /// # Notes
  /// - All trailing whitespaces are trimmed, empty lines are skipped.
  /// - This writes directly on the writer, avoiding extra allocation.
  fn render_lines(stdout: &mut dyn Write, lines: &Vec<&str>) -> () {
    for (index, line) in lines.iter().enumerate() {
      let trimmed_line = line.trim_end();

      if !trimmed_line.is_empty() {
        write!(
          stdout,
          "{goto}{text}",
          goto = cursor::Goto(1, index as u16 + 1),
          text = &trimmed_line,
        )
        .unwrap();
      }
    }
  }

  /// Render the Match's `text` field on provided writer.
  ///
  /// If a Mach is "focused", then it is rendered with a specific color.
  ///
  /// # Note
  /// This writes directly on the writer, avoiding extra allocation.
  fn render_matched_text(
    stdout: &mut dyn Write,
    text: &str,
    focused: bool,
    offset: (usize, usize),
    colors: &ViewColors,
  ) {
    // To help identify it, the match thas has focus is rendered with a dedicated color.
    let (text_fg_color, text_bg_color) = if focused {
      (&colors.focused_fg, &colors.focused_bg)
    } else {
      (&colors.match_fg, &colors.match_bg)
    };

    // Render just the Match's text on top of existing content.
    write!(
      stdout,
      "{goto}{bg_color}{fg_color}{text}{fg_reset}{bg_reset}",
      goto = cursor::Goto(offset.0 as u16 + 1, offset.1 as u16 + 1),
      fg_color = color::Fg(text_fg_color.as_ref()),
      bg_color = color::Bg(text_bg_color.as_ref()),
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
    stdout: &mut dyn Write,
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
        HintStyle::Underlined => {
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
        HintStyle::Surrounded(opening, closing) => {
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

  /// Render the view on the provided writer.
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
  fn render(&self, stdout: &mut dyn Write) -> () {
    write!(stdout, "{}", cursor::Hide).unwrap();

    // 1. Trim all lines and render non-empty ones.
    View::render_lines(stdout, self.state.lines);

    for (index, mat) in self.matches.iter().enumerate() {
      // 2. Render the match's text.

      // If multibyte characters occur before the hint (in the "prefix"), then
      // their compouding takes less space on screen when printed: for
      // instance ´ + e = é. Consequently the hint offset has to be adjusted
      // to the left.
      let offset_x = {
        let line = &self.state.lines[mat.y as usize];
        let prefix = &line[0..mat.x as usize];
        let adjust = prefix.len() - prefix.chars().count();
        (mat.x as usize) - (adjust)
      };
      let offset_y = mat.y as usize;

      let text = &mat.text;

      let focused = index == self.focus_index;

      View::render_matched_text(stdout, text, focused, (offset_x, offset_y), &self.rendering_colors);

      // 3. Render the hint (e.g. "eo") as an overlay on top of the rendered matched text,
      // aligned at its leading or the trailing edge.
      if let Some(ref hint) = mat.hint {
        let extra_offset = match self.hint_alignment {
          HintAlignment::Leading => 0,
          HintAlignment::Trailing => text.len() - hint.len(),
        };

        View::render_matched_hint(
          stdout,
          hint,
          (offset_x + extra_offset, offset_y),
          &self.rendering_colors,
          &self.hint_style,
        );
      }
    }

    stdout.flush().unwrap();
  }

  /// Listen to keys entered on stdin, moving focus accordingly, and selecting
  /// one or multiple matches.
  ///
  /// # Panics
  /// This function panics if termion cannot read the entered keys on stdin.
  /// This function also panics if the user types Insert on a line without hints.
  fn listen(&mut self, stdin: &mut dyn Read, stdout: &mut dyn Write) -> CaptureEvent {
    if self.matches.is_empty() {
      return CaptureEvent::Exit;
    }

    let mut chosen = vec![];
    let mut typed_hint: String = "".to_owned();
    let longest_hint = self
      .matches
      .iter()
      .filter_map(|m| m.hint.clone())
      .max_by(|x, y| x.len().cmp(&y.len()))
      .unwrap()
      .clone();

    self.render(stdout);

    loop {
      // This is an option of a result of a key... Let's pop error cases first.
      let next_key = stdin.keys().next();

      if next_key.is_none() {
        // Nothing in the buffer. Wait for a bit...
        std::thread::sleep(std::time::Duration::from_millis(100));
        continue;
      }

      let key_res = next_key.unwrap();
      if let Err(err) = key_res {
        // Termion not being able to read from stdin is an unrecoverable error.
        panic!(err);
      }

      match key_res.unwrap() {
        // Clears an ongoing multi-hint selection, or exit.
        Key::Esc => {
          if self.multi && !typed_hint.is_empty() {
            typed_hint.clear();
          } else {
            break;
          }
        }

        // In multi-selection mode, this appends the selected hint to the
        // vector of selections. In normal mode, this returns with the hint
        // selected.
        Key::Insert => match self.matches.get(self.focus_index) {
          Some(mat) => {
            chosen.push((mat.text.to_string(), false));

            if !self.multi {
              return CaptureEvent::Hint(chosen);
            }
          }
          None => panic!("Match not found?"),
        },

        // Move focus to next/prev match.
        Key::Up => self.prev(),
        Key::Down => self.next(),
        Key::Left => self.prev(),
        Key::Right => self.next(),

        // Pressing space finalizes an ongoing multi-hint selection (without
        // selecting the focused match). Pressing other characters attempts at
        // finding a match with a corresponding hint.
        Key::Char(ch) => {
          if ch == ' ' && self.multi {
            return CaptureEvent::Hint(chosen);
          }

          let key = ch.to_string();
          let lower_key = key.to_lowercase();

          typed_hint.push_str(&lower_key);

          // Find the match that corresponds to the entered key.
          let selection = self
            .matches
            .iter()
            // Avoid cloning typed_hint for comparison.
            .find(|&mat| mat.hint.as_deref().unwrap_or_default() == &typed_hint);

          match selection {
            Some(mat) => {
              chosen.push((mat.text.to_string(), key != lower_key));

              if self.multi {
                typed_hint.clear();
              } else {
                return CaptureEvent::Hint(chosen);
              }
            }
            None => {
              // TODO: use a Trie or another data structure to determine
              // if the entered key belongs to a longer hint.
              if !self.multi && typed_hint.len() >= longest_hint.len() {
                break;
              }
            }
          }
        }

        // Unknown keys are ignored.
        _ => (),
      }

      // Render on stdout if we did not exit earlier (move focus,
      // multi-selection).
      self.render(stdout);
    }

    CaptureEvent::Exit
  }

  pub fn present(&mut self) -> Vec<(String, bool)> {
    let mut stdin = async_stdin();
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

    let hints = match self.listen(&mut stdin, &mut stdout) {
      CaptureEvent::Exit => vec![],
      CaptureEvent::Hint(chosen) => chosen,
    };

    write!(stdout, "{}", cursor::Show).unwrap();

    hints
  }
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

    let mut writer = vec![];
    View::render_lines(&mut writer, &lines);

    let goto1 = cursor::Goto(1, 1);
    let goto2 = cursor::Goto(1, 2);
    let goto3 = cursor::Goto(1, 3);
    let goto6 = cursor::Goto(1, 6);
    assert_eq!(
      writer,
      format!(
        "{}some text{}* e006b06 - (12 days ago) swapper: Make quotes{}path: /usr/local/bin/git{}path: /usr/local/bin/cargo",
        goto1, goto2, goto3, goto6,
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
      focused_fg: Box::new(color::Red),
      focused_bg: Box::new(color::Blue),
      match_fg: Box::new(color::Green),
      match_bg: Box::new(color::Magenta),
      hint_fg: Box::new(color::Yellow),
      hint_bg: Box::new(color::Cyan),
    };

    let extra_offset = 0;
    let hint_style = Some(HintStyle::Underlined);

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
      focused_fg: Box::new(color::Red),
      focused_bg: Box::new(color::Blue),
      match_fg: Box::new(color::Green),
      match_bg: Box::new(color::Magenta),
      hint_fg: Box::new(color::Yellow),
      hint_bg: Box::new(color::Cyan),
    };

    let extra_offset = 0;
    let hint_style = Some(HintStyle::Surrounded('{', '}'));

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

    let custom_regexes = [].to_vec();
    let alphabet = alphabets::Alphabet("abcd".to_string());
    let mut state = state::State::new(&lines, &alphabet, &custom_regexes);
    let rendering_colors = ViewColors {
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
      state: &mut state,
      matches: vec![], // no matches
      focus_index: 0,
      multi: false,
      hint_alignment,
      rendering_colors: &rendering_colors,
      hint_style: None,
    };

    let mut writer = vec![];
    view.render(&mut writer);

    let hide = cursor::Hide;
    let goto1 = cursor::Goto(1, 1);
    let goto3 = cursor::Goto(1, 3);

    let expected = format!(
      "{hide}{goto1}lorem 127.0.0.1 lorem\
        {goto3}Barcelona https://en.wikipedia.org/wiki/Barcelona -",
      hide = hide,
      goto1 = goto1,
      goto3 = goto3,
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

    let custom_regexes = [].to_vec();
    let alphabet = alphabets::Alphabet("abcd".to_string());
    let mut state = state::State::new(&lines, &alphabet, &custom_regexes);
    let multi = false;
    let reversed = true;
    let unique = false;

    let rendering_colors = ViewColors {
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
      &mut state,
      multi,
      reversed,
      unique,
      hint_alignment,
      &rendering_colors,
      hint_style,
    );

    let mut writer = vec![];
    view.render(&mut writer);

    let expected_content = {
      let hide = cursor::Hide;
      let goto1 = cursor::Goto(1, 1);
      let goto3 = cursor::Goto(1, 3);

      format!(
        "{hide}{goto1}lorem 127.0.0.1 lorem\
        {goto3}Barcelona https://en.wikipedia.org/wiki/Barcelona -",
        hide = hide,
        goto1 = goto1,
        goto3 = goto3,
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

    let expected_match2_hint = {
      let goto11_3 = cursor::Goto(11, 3);

      format!(
        "{goto11_3}{hint_bg}{hint_fg}a{fg_reset}{bg_reset}",
        goto11_3 = goto11_3,
        hint_fg = color::Fg(rendering_colors.hint_fg.as_ref()),
        hint_bg = color::Bg(rendering_colors.hint_bg.as_ref()),
        fg_reset = color::Fg(color::Reset),
        bg_reset = color::Bg(color::Reset)
      )
    };

    let expected = [
      expected_content,
      expected_match1_text,
      expected_match1_hint,
      expected_match2_text,
      expected_match2_hint,
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
