use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal as Term, backend::CrosstermBackend};
use std::io::{Stdout, stdout};

pub struct Terminal {
    pub terminal: Term<CrosstermBackend<Stdout>>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(crossterm::event::DisableMouseCapture)?;
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Term::new(backend)?;
        terminal.clear()?;
        Ok(Terminal { terminal })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}
