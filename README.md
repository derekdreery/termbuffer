A library for building interactive apps with the terminal.

The usual way to interact with a terminal is to use ansi escape sequences.
These are commands like "use green foreground" or "move cursor up 1 place".
This library presents a blank buffer for the user to draw on, and converts this
buffer into the minimum (aspirational) number of bytes to send to the terminal.
This is sometimes referred to as immediate mode drawing, where you draw
everything each frame.

The library also provides an interator over all input events received since the
last request for the iterator.

# Example

```rust
use termbuffer::{App, Event, Key, char, Color};
use std::time::{Duration, Instant};
use std::thread;

const FRAME_TIME: Duration = Duration::from_millis(1000 / 60); // 60 fps

fn main() {
    let mut shutdown = false;

    // Currently there are no options, but I'm planning to add for things like
    // what color to clear the screen to, etc.
    let mut app = App::builder().build().unwrap();
    // As this counter is incremented, we will move along the rows.
    let mut counter = 0;
    loop {
        if shutdown {
            break;
        }
        let time_start = Instant::now();
        {
            // Call draw when you are ready to start rendering the next frame.
            let mut draw = app.draw();
            // The draw object contains the new number of rows and columns
            // (this will change if the user resizes the terminal).
            let cols = draw.columns();
            let rows = draw.rows();
            // Math to convert counter to position.
            let col = counter / rows;
            let row = counter % rows;
            // We set all the characters we want.
            draw.set(row, col, char!('.', Color::Default, Color::Red));
            counter = (counter + 1) % (cols * rows);
        }
        // Call app.events to pump the event loop.
        for evt in app.events() {
            match evt.unwrap() {
                Event::Key(Key::Char('q')) => {
                    shutdown = true;
                }
                _ => ()
            }
        }
        let time_end = Instant::now();
        // Sleep for the remainder of the frame.
        if time_end < time_start + FRAME_TIME {
            thread::sleep(FRAME_TIME - (time_end - time_start));
        }
    }
}
```
