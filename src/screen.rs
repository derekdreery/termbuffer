use std::io::{self, Write};
use std::ops::{Deref, DerefMut};
use std::mem;

#[derive(Debug)]
pub(crate) struct Screen {
    pub(crate) previous: Frame,
    pub(crate) next: Frame,
}

impl Screen {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Screen {
            previous: Frame::new(width, height),
            next: Frame::new(width, height),
        }
    }
    pub(crate) fn prepare_next_frame(&mut self, width: usize, height: usize) {
        mem::swap(&mut self.next, &mut self.previous);
        self.next.reset(width, height);
    }

    /// Render the frame to the terminal
    pub(crate) fn render(&self, writer: &mut impl Write) -> io::Result<()> {
        if self.next.dims() != self.previous.dims() {
            // We need to redraw
            self.redraw(writer)
        } else {
            // We can do incremental update
            self.redraw_diff(writer)
        }
    }

    pub(crate) fn redraw(&self, writer: &mut impl Write) -> io::Result<()> {
        use termion::{
            cursor::{Right, Goto},
        };
        write!(writer, "{}", termion::clear::All)?;
        assert!(self.next.rows < u16::max_value().into(), "rows must fit in u16");
        for row in 0..self.next.rows {
            for col in 0..self.next.cols {
                write!(writer, "{}", Goto((row as u16) + 1, (col as u16) + 1))?;
                let current = self.next.get(row, col);
                // Change color if we need to.
                if let Some((prev_row, prev_col)) = self.next.prev_row_col(row, col) {
                    let prev = self.next.get(prev_row, prev_col);
                    if prev.color_fg != current.color_fg {
                        current.write_fg(writer)?;
                    }
                    if prev.color_bg != current.color_bg {
                        current.write_bg(writer)?;
                    }
                } else {
                    current.write_fg(writer)?;
                    current.write_bg(writer)?;
                }
                write!(writer, "{}", current.glyph)?;
            }
        }
        Ok(())
    }

    pub(crate) fn redraw_diff(&self, writer: &mut impl Write) -> io::Result<()> {
        use termion::{
            cursor::{Right, Goto},
        };
        assert!(self.next.rows < u16::max_value().into(), "rows must fit in u16");
        let mut prev_fg = Color::default();
        let mut prev_bg = Color::default();
        prev_fg.write_fg(writer)?;
        prev_bg.write_bg(writer)?;
        for row in 0..self.next.rows {
            for col in 0..self.next.cols {
                let next = self.next.get(row, col);
                let prev = self.previous.get(row, col);
                if next == prev {
                    continue
                }
                write!(writer, "{}", Goto((row as u16) + 1, (col as u16) + 1))?;
                // Change color if we need to.
                if next.color_fg != prev_fg {
                    next.write_fg(writer)?;
                    prev_fg = next.color_fg
                }
                if next.color_bg != prev_bg {
                    next.write_bg(writer)?;
                    prev_bg = next.color_bg
                }
                write!(writer, "{}", next.glyph)?;
            }
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Frame {
    rows: usize,
    cols: usize,
    buffer: Vec<Char>
}

impl Frame {
    fn new(rows: usize, cols: usize) -> Frame {
        Frame {
            rows,
            cols,
            buffer: vec![Default::default(); rows * cols]
        }
    }

    /// Reset the size and clear the contents of the screen
    fn reset(&mut self, rows: usize, cols: usize) {
        self.buffer.clear();
        self.rows = rows;
        self.cols = cols;
        for _ in 0..(rows * cols) {
            self.buffer.push(Default::default());
        }
    }

    /// The number of rows on the screen.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// The number of columns on the screen.
    pub fn columns(&self) -> usize {
        self.cols
    }

    /// Private shorthand for comparing dims.
    fn dims(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Will panic if the row or column is out of bounds.
    pub fn set(&mut self, row: usize, col: usize, ch: Char) {
        self.check_dims(row, col);
        self.buffer[col * self.rows + row] = ch;
    }

    pub fn get(&self, row: usize, col: usize) -> Char {
        self.check_dims(row, col);
        self.buffer[col * self.rows + row]
    }

    fn prev_row_col(&self, row: usize, col: usize) -> Option<(usize, usize)> {
        if row == 0 && col == 0 {
            None
        } else {
            match col {
                0 => Some((row - 1, self.cols - 1)),
                n => Some((row, n - 1)),
            }
        }
    }

    fn check_dims(&self, row: usize, col: usize) {
        if row >= self.rows {
            panic!("Row {} is out of bounds (number of rows: {})", row, self.rows);
        }
        if col >= self.cols {
            panic!("Column {} is out of bounds (number of columns: {})", col, self.cols);
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Char {
    pub glyph: char,
    pub color_fg: Color,
    pub color_bg: Color,
}

impl Char {
    pub fn new(glyph: char) -> Char {
        Char {
            glyph,
            color_fg: Color::default(),
            color_bg: Color::default(),
        }
    }

    pub fn write_fg(&self, writer: &mut impl Write) -> io::Result<()> {
        self.color_fg.write_fg(writer)
    }

    pub fn write_bg(&self, writer: &mut impl Write) -> io::Result<()> {
        self.color_bg.write_bg(writer)
    }
}

impl Default for Char {
    fn default() -> Self {
        Char {
            glyph: ' ',
            color_fg: Color::default(),
            color_bg: Color::default()
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Color {
    Default,
    Black,
    Blue,
    Cyan,
    LightBlack,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
    Magenta,
    Red,
    Rgb(u8, u8, u8),
    White,
    Yellow,
}

impl Color {
    fn write_fg(&self, writer: &mut impl Write) -> io::Result<()> {
        use termion::color;
        match self {
            Color::Default => write!(writer, "{}", color::Fg(color::Reset)),
            Color::Black => write!(writer, "{}", color::Fg(color::Black)),
            Color::Blue => write!(writer, "{}", color::Fg(color::Blue)),
            Color::Cyan => write!(writer, "{}", color::Fg(color::Cyan)),
            Color::LightBlack => write!(writer, "{}", color::Fg(color::LightBlack)),
            Color::LightBlue => write!(writer, "{}", color::Fg(color::LightBlue)),
            Color::LightCyan => write!(writer, "{}", color::Fg(color::LightCyan)),
            Color::LightGreen => write!(writer, "{}", color::Fg(color::LightGreen)),
            Color::LightMagenta => write!(writer, "{}", color::Fg(color::LightMagenta)),
            Color::LightRed => write!(writer, "{}", color::Fg(color::LightRed)),
            Color::LightWhite => write!(writer, "{}", color::Fg(color::LightWhite)),
            Color::LightYellow => write!(writer, "{}", color::Fg(color::LightYellow)),
            Color::Magenta => write!(writer, "{}", color::Fg(color::Magenta)),
            Color::Red => write!(writer, "{}", color::Fg(color::Red)),
            Color::Rgb(r, g, b) => write!(writer, "{}", color::Fg(color::Rgb(*r, *g, *b))),
            Color::White => write!(writer, "{}", color::Fg(color::White)),
            Color::Yellow => write!(writer, "{}", color::Fg(color::Yellow)),
        }
    }
    fn write_bg(&self, writer: &mut impl Write) -> io::Result<()> {
        use termion::color;
        match self {
            Color::Default => write!(writer, "{}", color::Bg(color::Reset)),
            Color::Black => write!(writer, "{}", color::Bg(color::Black)),
            Color::Blue => write!(writer, "{}", color::Bg(color::Blue)),
            Color::Cyan => write!(writer, "{}", color::Bg(color::Cyan)),
            Color::LightBlack => write!(writer, "{}", color::Bg(color::LightBlack)),
            Color::LightBlue => write!(writer, "{}", color::Bg(color::LightBlue)),
            Color::LightCyan => write!(writer, "{}", color::Bg(color::LightCyan)),
            Color::LightGreen => write!(writer, "{}", color::Bg(color::LightGreen)),
            Color::LightMagenta => write!(writer, "{}", color::Bg(color::LightMagenta)),
            Color::LightRed => write!(writer, "{}", color::Bg(color::LightRed)),
            Color::LightWhite => write!(writer, "{}", color::Bg(color::LightWhite)),
            Color::LightYellow => write!(writer, "{}", color::Bg(color::LightYellow)),
            Color::Magenta => write!(writer, "{}", color::Bg(color::Magenta)),
            Color::Red => write!(writer, "{}", color::Bg(color::Red)),
            Color::Rgb(r, g, b) => write!(writer, "{}", color::Bg(color::Rgb(*r, *g, *b))),
            Color::White => write!(writer, "{}", color::Bg(color::White)),
            Color::Yellow => write!(writer, "{}", color::Bg(color::Yellow)),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Default
    }
}

#[macro_export]
macro_rules! char {
    () => {
        $crate::Char::default()
    };
    ($glyph:expr) => {
        $crate::Char::new($glyph)
    };
    ($glyph:expr, $fg:expr) => {
        $crate::Char {
            glyph: $glyph,
            color_fg: $fg,
            color_bg: Color::default(),
        }
    };
    ($glyph:expr, $fg:expr, $bg:expr) => {
        $crate::Char {
            glyph: $glyph,
            color_fg: $fg,
            color_bg: $bg,
        }
    };
}
