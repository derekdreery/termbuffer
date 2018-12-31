use termbuffer::{App, Event, Key, char, Color};
use std::time::{Duration, Instant};
use std::thread;

const FRAME_TIME: Duration = Duration::from_millis(1000 / 60); // 60 fps

fn main() {
    let mut shutdown = false;

    let mut app = App::builder().build().unwrap();
    let mut counter = 0;
    loop {
        if shutdown {
            break;
        }
        let time_start = Instant::now();
        {
            let mut draw = app.draw();
            let cols = draw.columns();
            let rows = draw.rows();
            let col = counter / rows;
            let row = counter % rows;
            draw.set(row, col, char!('.', Color::Default, Color::Red));
            counter = (counter + 1) % (cols * rows);
        }
        for evt in app.events() {
            match evt.unwrap() {
                Event::Key(Key::Char('q')) => {
                    shutdown = true;
                }
                _ => ()
            }
        }
        let time_end = Instant::now();
        if time_end < time_start + FRAME_TIME {
            thread::sleep(FRAME_TIME - (time_end - time_start));
        }
    }
}
