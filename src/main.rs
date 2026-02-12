#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]
mod editor;
use editor::Editor;
use std::io::stdout;

fn main() {
    // Install a custom panic hook to ensure the terminal is restored even if we panic.
    // Without this, a panic would leave the terminal in raw mode and on the alternate screen.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = crossterm::execute!(
            stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        let _ = crossterm::terminal::disable_raw_mode();
        original_hook(panic_info);
    }));

    match Editor::new() {
        Ok(mut editor) => editor.run(),
        Err(err) => {
            // Terminal failed to initialize. Since we may be partially initialized,
            // attempt cleanup before reporting.
            let _ = crossterm::terminal::disable_raw_mode();
            eprintln!("Failed to initialize editor: {err}");
        }
    }
}
