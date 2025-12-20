use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal as RatatuiTerminal};
use std::io::{self, Stdout};

pub struct Terminal {
    pub backend: RatatuiTerminal<CrosstermBackend<Stdout>>,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = RatatuiTerminal::new(backend)?;
        Ok(Self { backend: terminal })
    }

    pub fn stop(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.backend.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.backend.show_cursor()?;
        Ok(())
    }
}
