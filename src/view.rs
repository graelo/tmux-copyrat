use super::*;
use std::char;
use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{color, cursor};

pub struct View<'a> {
  state: &'a mut state::State<'a>,
  skip: usize,
  multi: bool,
  contrast: bool,
  position: &'a str,
  matches: Vec<state::Match<'a>>,
  select_foreground_color: Box<&'a dyn color::Color>,
  select_background_color: Box<&'a dyn color::Color>,
  foreground_color: Box<&'a dyn color::Color>,
  background_color: Box<&'a dyn color::Color>,
  hint_background_color: Box<&'a dyn color::Color>,
  hint_foreground_color: Box<&'a dyn color::Color>,
}

enum CaptureEvent {
  Exit,
  Hint(Vec<(String, bool)>),
}

impl<'a> View<'a> {
  pub fn new(
    state: &'a mut state::State<'a>,
    multi: bool,
    reverse: bool,
    unique: bool,
    contrast: bool,
    position: &'a str,
    select_foreground_color: Box<&'a dyn color::Color>,
    select_background_color: Box<&'a dyn color::Color>,
    foreground_color: Box<&'a dyn color::Color>,
    background_color: Box<&'a dyn color::Color>,
    hint_foreground_color: Box<&'a dyn color::Color>,
    hint_background_color: Box<&'a dyn color::Color>,
  ) -> View<'a> {
    let matches = state.matches(reverse, unique);
    let skip = if reverse { matches.len() - 1 } else { 0 };

    View {
      state,
      skip,
      multi,
      contrast,
      position,
      matches,
      select_foreground_color,
      select_background_color,
      foreground_color,
      background_color,
      hint_foreground_color,
      hint_background_color,
    }
  }

  /// Move focus onto the previous hint.
  pub fn prev(&mut self) {
    if self.skip > 0 {
      self.skip -= 1;
    }
  }

  /// Move focus onto the next hint.
  pub fn next(&mut self) {
    if self.skip < self.matches.len() - 1 {
      self.skip += 1;
    }
  }

  // /// TODO remove
  // fn make_hint_text(&self, hint: &str) -> String {
  //   if self.contrast {
  //     format!("[{}]", hint)
  //   } else {
  //     hint.to_string()
  //   }
  // }

  /// Render the view on stdout.
  fn render(&self, stdout: &mut dyn Write) -> () {
    write!(stdout, "{}", cursor::Hide).unwrap();

    // Trim all lines and render non-empty ones.
    for (index, line) in self.state.lines.iter().enumerate() {
      // remove trailing whitespaces
      let cleaned_line = line.trim_end_matches(|c: char| c.is_whitespace());

      if cleaned_line.is_empty() {
        continue; // Don't render empty lines.
      }

      // let text = self.make_hint_text(line);
      // print!(
      write!(
        stdout,
        "{goto}{text}",
        goto = cursor::Goto(1, index as u16 + 1),
        text = &cleaned_line,
      )
      .unwrap();
    }

    // let focused = self.matches.get(self.skip);

    for (index, mat) in self.matches.iter().enumerate() {
      // 1. Render the match's text.
      //

      // To help identify it, the match thas has focus is rendered with a dedicated color.
      // let (text_fg_color, text_bg_color) = if focused == Some(mat) {
      let (text_fg_color, text_bg_color) = if index == self.skip {
        (&self.select_foreground_color, &self.select_background_color)
      } else {
        (&self.foreground_color, &self.background_color)
      };

      // If multibyte characters occur before the hint (in the "prefix"), then
      // their compouding takes less space on screen when printed: for
      // instance ´ + e = é. Consequently the hint offset has to be adjusted
      // to the left.
      let line = &self.state.lines[mat.y as usize];
      let prefix = &line[0..mat.x as usize];
      let adjust = prefix.len() - prefix.chars().count();
      let offset = (mat.x as u16) - (adjust as u16);
      let text = &mat.text; //self.make_hint_text(mat.text);

      // Render just the match's text on top of existing content.
      write!(
        stdout,
        "{goto}{bg_color}{fg_color}{text}{fg_reset}{bg_reset}",
        goto = cursor::Goto(offset + 1, mat.y as u16 + 1),
        fg_color = color::Fg(**text_fg_color),
        bg_color = color::Bg(**text_bg_color),
        fg_reset = color::Fg(color::Reset),
        bg_reset = color::Bg(color::Reset),
        text = &text,
      )
      .unwrap();

      // 2. Render the hint (e.g. ";k") on top of the text at the beginning or the end.
      //
      if let Some(ref hint) = mat.hint {
        let extra_offset = if self.position == "left" {
          0
        } else {
          text.len() - hint.len()
        };

        write!(
          stdout,
          "{goto}{bg_color}{fg_color}{hint}{fg_reset}{bg_reset}",
          goto = cursor::Goto(offset + extra_offset as u16 + 1, mat.y as u16 + 1),
          fg_color = color::Fg(*self.hint_foreground_color),
          bg_color = color::Bg(*self.hint_background_color),
          fg_reset = color::Fg(color::Reset),
          bg_reset = color::Bg(color::Reset),
          hint = hint,
        )
        .unwrap();
      }
    }

    stdout.flush().unwrap();
  }

  /// Listen to keys entered on stdin
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

        // TODO: What does this do?
        Key::Insert => match self.matches.iter().enumerate().find(|&(idx, _)| idx == self.skip) {
          Some((_idx, mtch)) => {
            chosen.push((mtch.text.to_string(), false));

            if !self.multi {
              return CaptureEvent::Hint(chosen);
            }
          }
          None => panic!("Match not found?"),
        },

        // Move focus to next/prev hint.
        Key::Up => self.prev(),
        Key::Down => self.next(),
        Key::Left => self.prev(),
        Key::Right => self.next(),

        // Pressing space finalizes an ongoing multi-hint selection.
        // Others characters attempt the corresponding hint.
        Key::Char(ch) => {
          if ch == ' ' && self.multi {
            return CaptureEvent::Hint(chosen);
          }

          let key = ch.to_string();
          let lower_key = key.to_lowercase();

          typed_hint.push_str(&lower_key);

          let selection = self.matches.iter().find(|&mtch| mtch.hint == Some(typed_hint.clone()));

          match selection {
            Some(mtch) => {
              chosen.push((mtch.text.to_string(), key != lower_key));

              if self.multi {
                typed_hint.clear();
              } else {
                return CaptureEvent::Hint(chosen);
              }
            }
            None => {
              if !self.multi && typed_hint.len() >= longest_hint.len() {
                break;
              }
            }
          }
        }

        // Unknown keys are ignored.
        _ => (),
      }

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

  fn split(output: &str) -> Vec<&str> {
    output.split("\n").collect::<Vec<&str>>()
  }

  #[test]
  fn hint_text() {
    let lines = split("lorem 127.0.0.1 lorem");
    let custom = [].to_vec();
    let mut state = state::State::new(&lines, "abcd", &custom);
    let mut view = View {
      state: &mut state,
      skip: 0,
      multi: false,
      contrast: false,
      position: &"",
      matches: vec![],
      select_foreground_color: colors::get_color("default"),
      select_background_color: colors::get_color("default"),
      foreground_color: colors::get_color("default"),
      background_color: colors::get_color("default"),
      hint_background_color: colors::get_color("default"),
      hint_foreground_color: colors::get_color("default"),
    };

    // let result = view.make_hint_text("a");
    // assert_eq!(result, "a".to_string());

    // view.contrast = true;
    // let result = view.make_hint_text("a");
    // assert_eq!(result, "[a]".to_string());
  }
}
