use std::ops::Range;

/// Represents a Window on the underlying buffer.
///
/// The start index points at the first line, the end index points at the line
/// after the last one, so that `start..end` represents a range.
pub struct Window {
    range: Range<usize>,
    height: usize,
    total_height: usize,
    // buffer: &'a [&'a str],
}

impl Window {
    /// Initialize a new Window.
    ///
    /// # Panics
    ///
    /// This function panics if the end of the window (`start + height`) is
    /// larger than `total_height`.
    pub fn new(start: usize, height: usize, total_height: usize) -> Window {
        if start + height > total_height {
            panic!("Window height must be larger than total_height.");
        }

        Window {
            range: Range {
                start,
                end: start + height,
            },
            height,
            total_height,
        }
    }

    /// Move the Window by some amount of lines, respecting the limits of the
    /// total underlying buffer which has total_height lines.
    pub fn move_by(&mut self, amount: isize) {
        let candidate_start = (self.range.start as isize) + amount;
        if candidate_start < 0 {
            self.range = Range {
                start: 0,
                end: self.height,
            }
        } else if candidate_start as usize + self.height > self.total_height {
            let start = self.total_height - self.height;
            self.range = Range {
                start,
                end: start + self.height,
            }
        } else {
            self.range = Range {
                start: candidate_start as usize,
                end: candidate_start as usize + self.height,
            }
        }
    }

    /// Center the Window around the provided line index, respecting the
    /// limits of the underlying buffer which has total_height lines.
    pub fn center_on(&mut self, line_index: usize) {
        let candidate_start = line_index as isize - (self.height as isize) / 2;
        let delta = candidate_start as isize - self.range.start as isize;
        self.move_by(delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ok() {
        let w = Window::new(0, 100, 200);
        assert_eq!(w.range.start, 0);
        assert_eq!(w.range.end, 100);
        assert_eq!(w.height, 100);
        assert_eq!(w.total_height, 200);
    }

    #[test]
    fn move_by_allowed_amount() {
        let mut w = Window {
            range: 0..100,
            height: 100,
            total_height: 200,
        };

        w.move_by(50);
        assert_eq!(w.range.start, 50);
        assert_eq!(w.range.end, 150);
        assert_eq!(w.height, 100);

        w.move_by(50);
        assert_eq!(w.range.start, 100);
        assert_eq!(w.range.end, 200);
        assert_eq!(w.height, 100);

        w.move_by(-50);
        assert_eq!(w.range.start, 50);
        assert_eq!(w.range.end, 150);
        assert_eq!(w.height, 100);
    }

    #[test]
    fn move_by_clamped() {
        let mut w = Window {
            range: 0..100,
            height: 100,
            total_height: 200,
        };

        w.move_by(150);
        assert_eq!(w.range.start, 100);
        assert_eq!(w.range.end, 200);
        assert_eq!(w.height, 100);

        w.move_by(150);
        assert_eq!(w.range.start, 100);
        assert_eq!(w.range.end, 200);
        assert_eq!(w.height, 100);

        w.move_by(-50);
        assert_eq!(w.range.start, 50);
        assert_eq!(w.range.end, 150);
        assert_eq!(w.height, 100);

        w.move_by(-50);
        assert_eq!(w.range.start, 0);
        assert_eq!(w.range.end, 100);
        assert_eq!(w.height, 100);

        w.move_by(-50);
        assert_eq!(w.range.start, 0);
        assert_eq!(w.range.end, 100);
        assert_eq!(w.height, 100);
    }

    #[test]
    fn center_on_allowed() {
        let mut w = Window {
            range: 0..100,
            height: 100,
            total_height: 200,
        };

        w.center_on(100);
        assert_eq!(w.range.start, 50);
        assert_eq!(w.range.end, 150);
        assert_eq!(w.height, 100);

        w.center_on(50);
        assert_eq!(w.range.start, 0);
        assert_eq!(w.range.end, 100);
        assert_eq!(w.height, 100);
    }

    #[test]
    fn center_on_clamped() {
        let mut w = Window {
            range: 0..100,
            height: 100,
            total_height: 200,
        };

        w.center_on(1);
        assert_eq!(w.range.start, 0);
        assert_eq!(w.range.end, 100);
        assert_eq!(w.height, 100);

        w.center_on(199);
        assert_eq!(w.range.start, 100);
        assert_eq!(w.range.end, 200);
        assert_eq!(w.height, 100);
    }
}
