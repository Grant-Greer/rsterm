use core::cmp::min;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read,
};

use std::io::Error;
mod terminal;
mod view;
use terminal::{Position, Size, Terminal};

use view::View;

#[derive(Copy, Clone, Default)]
struct Location {
    x: usize,
    y: usize,
}

pub struct Editor {
    should_quit: bool,
    location: Location,
    view: View,
    _terminal: Terminal,
}

impl Editor {
    /// Creates a new Editor, initializing the terminal (raw mode + alternate screen).
    /// Returns an error if terminal initialization fails.
    pub fn new() -> Result<Self, Error> {
        let terminal = Terminal::new()?;
        Ok(Self {
            should_quit: false,
            location: Location::default(),
            view: View::default(),
            _terminal: terminal,
        })
    }

    pub fn run(&mut self) {
        let result = self.repl();
        if let Err(err) = result {
            // In debug builds, panic so the error is visible during development.
            // In release builds, silently continue — Drop will clean up the terminal.
            debug_assert!(false, "Error in editor main loop: {err}");
        }
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(event);
        }
        Ok(())
    }

    fn move_point(&mut self, key_code: KeyCode) {
        let Location { mut x, mut y } = self.location;
        // Handle Terminal::size() locally: if we can't read the terminal size,
        // we simply skip the movement rather than propagating the error.
        let Ok(Size { height, width }) = Terminal::size() else {
            return;
        };
        match key_code {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                y = min(height.saturating_sub(1), y.saturating_add(1));
            }
            KeyCode::Left => {
                x = x.saturating_sub(1);
            }
            KeyCode::Right => {
                x = min(width.saturating_sub(1), x.saturating_add(1));
            }
            KeyCode::PageUp => {
                y = 0;
            }
            KeyCode::PageDown => {
                y = height.saturating_sub(1);
            }
            KeyCode::Home => {
                x = 0;
            }
            KeyCode::End => {
                x = width.saturating_sub(1);
            }
            _ => (),
        }
        self.location = Location { x, y };
    }

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                modifiers,
                ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                (
                    KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::PageDown
                    | KeyCode::PageUp
                    | KeyCode::End
                    | KeyCode::Home,
                    _,
                ) => {
                    self.move_point(code);
                }
                _ => {}
            },
            Event::Resize(width_u16, height_u16) => {
                #[allow(clippy::as_conversions)]
                let height = height_u16 as usize;

                #[allow(clippy::as_conversions)]
                let width = width_u16 as usize;
                self.view.resize(Size { height, width });
            }
            _ => {}
        }
    }

    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        Terminal::move_caret_to(Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            Terminal::move_caret_to(Position {
                col: self.location.x,
                row: self.location.y,
            })?;
        }

        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
}
