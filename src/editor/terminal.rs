use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};
use crossterm::{Command, queue};
use std::io::{Error, Write, stdout};

#[derive(Default, Copy, Clone)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

/// Manages terminal state including raw mode and alternate screen.
/// Implements `Drop` to ensure cleanup even on unexpected exits.
pub struct Terminal;

impl Terminal {
    /// Creates a new Terminal, entering raw mode and the alternate screen.
    /// Use this instead of `Default` since initialization has side effects.
    pub fn new() -> Result<Self, Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::clear_screen()?;
        Self::execute()?;
        Ok(Self)
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        debug_assert!(
            u16::try_from(position.col).is_ok(),
            "Column position {col} exceeds u16::MAX",
            col = position.col
        );
        debug_assert!(
            u16::try_from(position.row).is_ok(),
            "Row position {row} exceeds u16::MAX",
            row = position.row
        );
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))?;
        Ok(())
    }

    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }

    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)?;
        Ok(())
    }

    pub fn print(string: &str) -> Result<(), Error> {
        Self::queue_command(Print(string))?;
        Ok(())
    }

    pub fn size() -> Result<Size, Error> {
        let (width_u16, height_u16) = size()?;

        #[allow(clippy::as_conversions)]
        let height = height_u16 as usize;

        #[allow(clippy::as_conversions)]
        let width = width_u16 as usize;
        Ok(Size { height, width })
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(EnterAlternateScreen)?;
        Ok(())
    }

    fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(LeaveAlternateScreen)?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_default_is_zero() {
        let size = Size::default();
        assert_eq!(size.height, 0);
        assert_eq!(size.width, 0);
    }

    #[test]
    fn position_default_is_zero() {
        let pos = Position::default();
        assert_eq!(pos.col, 0);
        assert_eq!(pos.row, 0);
    }

    #[test]
    fn size_stores_values() {
        let size = Size {
            height: 24,
            width: 80,
        };
        assert_eq!(size.height, 24);
        assert_eq!(size.width, 80);
    }

    #[test]
    fn position_stores_values() {
        let pos = Position { col: 10, row: 5 };
        assert_eq!(pos.col, 10);
        assert_eq!(pos.row, 5);
    }

    #[test]
    fn size_is_copy() {
        let size = Size {
            height: 24,
            width: 80,
        };
        let size2 = size;
        assert_eq!(size.height, size2.height);
    }

    #[test]
    fn position_is_copy() {
        let pos = Position { col: 10, row: 5 };
        let pos2 = pos;
        assert_eq!(pos.col, pos2.col);
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Best-effort cleanup. Each step is independent so we attempt all of them
        // even if earlier ones fail. We must not panic here.
        let _ = Self::leave_alternate_screen();
        let _ = Self::show_caret();
        let _ = Self::execute();
        let _ = disable_raw_mode();
    }
}
