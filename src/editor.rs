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
        // Handle Terminal::size() locally: if we can't read the terminal size,
        // we simply skip the movement rather than propagating the error.
        let Ok(size) = Terminal::size() else {
            return;
        };
        self.location = Self::calculate_movement(self.location, key_code, size);
    }

    fn calculate_movement(location: Location, key_code: KeyCode, size: Size) -> Location {
        let Location { mut x, mut y } = location;
        let Size { height, width } = size;
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
        Location { x, y }
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SIZE: Size = Size {
        height: 24,
        width: 80,
    };

    fn loc(x: usize, y: usize) -> Location {
        Location { x, y }
    }

    #[test]
    fn move_up_from_origin_stays_at_zero() {
        let result = Editor::calculate_movement(loc(0, 0), KeyCode::Up, TEST_SIZE);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn move_up_decrements_y() {
        let result = Editor::calculate_movement(loc(5, 10), KeyCode::Up, TEST_SIZE);
        assert_eq!(result.y, 9);
        assert_eq!(result.x, 5);
    }

    #[test]
    fn move_down_increments_y() {
        let result = Editor::calculate_movement(loc(5, 10), KeyCode::Down, TEST_SIZE);
        assert_eq!(result.y, 11);
        assert_eq!(result.x, 5);
    }

    #[test]
    fn move_down_clamps_to_bottom() {
        let result = Editor::calculate_movement(loc(0, 23), KeyCode::Down, TEST_SIZE);
        assert_eq!(result.y, 23);
    }

    #[test]
    fn move_left_from_origin_stays_at_zero() {
        let result = Editor::calculate_movement(loc(0, 0), KeyCode::Left, TEST_SIZE);
        assert_eq!(result.x, 0);
    }

    #[test]
    fn move_left_decrements_x() {
        let result = Editor::calculate_movement(loc(5, 0), KeyCode::Left, TEST_SIZE);
        assert_eq!(result.x, 4);
    }

    #[test]
    fn move_right_increments_x() {
        let result = Editor::calculate_movement(loc(5, 0), KeyCode::Right, TEST_SIZE);
        assert_eq!(result.x, 6);
    }

    #[test]
    fn move_right_clamps_to_width() {
        let result = Editor::calculate_movement(loc(79, 0), KeyCode::Right, TEST_SIZE);
        assert_eq!(result.x, 79);
    }

    #[test]
    fn page_up_jumps_to_top() {
        let result = Editor::calculate_movement(loc(5, 15), KeyCode::PageUp, TEST_SIZE);
        assert_eq!(result.y, 0);
        assert_eq!(result.x, 5);
    }

    #[test]
    fn page_down_jumps_to_bottom() {
        let result = Editor::calculate_movement(loc(5, 0), KeyCode::PageDown, TEST_SIZE);
        assert_eq!(result.y, 23);
        assert_eq!(result.x, 5);
    }

    #[test]
    fn home_jumps_to_start_of_line() {
        let result = Editor::calculate_movement(loc(40, 10), KeyCode::Home, TEST_SIZE);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 10);
    }

    #[test]
    fn end_jumps_to_end_of_line() {
        let result = Editor::calculate_movement(loc(0, 10), KeyCode::End, TEST_SIZE);
        assert_eq!(result.x, 79);
        assert_eq!(result.y, 10);
    }

    #[test]
    fn unhandled_key_does_not_move() {
        let result =
            Editor::calculate_movement(loc(5, 10), KeyCode::Char('a'), TEST_SIZE);
        assert_eq!(result.x, 5);
        assert_eq!(result.y, 10);
    }

    #[test]
    fn movement_with_zero_size_terminal() {
        let zero_size = Size {
            height: 0,
            width: 0,
        };
        let result = Editor::calculate_movement(loc(0, 0), KeyCode::Down, zero_size);
        assert_eq!(result.y, 0);
        let result = Editor::calculate_movement(loc(0, 0), KeyCode::Right, zero_size);
        assert_eq!(result.x, 0);
    }
}
