
use std::{
    io::{self, Read, Write},
    thread,
    time::{Instant, Duration},
    ops::{Deref, DerefMut},
};
use termion::{
    raw::IntoRawMode,
    input::{TermRead, Events},
    terminal_size, async_stdin, clear, cursor,
    raw::RawTerminal,
    AsyncReader,
};
pub use crate::screen::{Frame, Char, Color};
pub use termion::event::{Event, Key, MouseButton, MouseEvent};

mod screen;

pub struct App {
    output: RawTerminal<io::Stdout>,
    input: Events<AsyncReader>,
    screen: screen::Screen,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn draw<'a>(&'a mut self) -> Draw<'a> {
        let (width, height) = terminal_size().unwrap();
        let (width, height) = (width as usize, height as usize);
        self.screen.prepare_next_frame(width, height);
        Draw {
            output: &mut self.output,
            screen: &mut self.screen
        }
    }

    pub fn events<'a>(&'a mut self) -> &'a mut (impl Iterator<Item=io::Result<Event>> + 'a) {
        &mut self.input
    }
}

impl Drop for App {
    fn drop(&mut self) {
        use termion::color;
        // The best we can do here is to ignore errors.
        let _ = write!(self.output, "{}{}{}{}{}",
                       color::Fg(color::Reset),
                       color::Bg(color::Reset),
                       clear::All,
                       cursor::Goto(1, 1),
                       cursor::Show);
    }
}

#[derive(Debug, Clone)]
pub struct AppBuilder {
}

impl AppBuilder {
    pub fn build(self) -> io::Result<App> {
        let mut output = io::stdout().into_raw_mode()?;
        write!(output, "{}{}", clear::All, cursor::Hide)?;
        let mut input = async_stdin().events();
        let (width, height) = terminal_size()?;
        let (width, height) = (width as usize, height as usize);
        output.flush()?;
        Ok(App {
            input,
            output,
            screen: screen::Screen::new(width, height),
        })
    }
}

impl Default for AppBuilder {
    fn default() -> AppBuilder {
        AppBuilder {
        }
    }
}

pub struct Draw<'a> {
    screen: &'a mut screen::Screen,
    output: &'a mut RawTerminal<io::Stdout>,
}

impl<'a> Deref for Draw<'a> {
    type Target = Frame;
    fn deref(&self) -> &Frame {
        &self.screen.next
    }
}

impl<'a> DerefMut for Draw<'a> {
    fn deref_mut(&mut self) -> &mut Frame {
        &mut self.screen.next
    }
}

impl<'a> Drop for Draw<'a> {
    fn drop(&mut self) {
        self.screen.render(&mut self.output.lock()).unwrap();
        self.output.flush().unwrap();
    }
}

