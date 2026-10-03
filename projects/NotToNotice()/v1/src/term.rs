//! Terminal lifecycle for the interactive live player.

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use std::io::Write;

pub struct Term {
    out: std::io::Stdout,
}

impl Term {
    pub fn init() -> Result<Term> {
        let mut out = std::io::stdout();
        terminal::enable_raw_mode()?;
        crossterm::execute!(out, terminal::EnterAlternateScreen)?;
        crossterm::execute!(
            out,
            crossterm::style::SetForegroundColor(crossterm::style::Color::Reset),
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
        )?;
        crossterm::execute!(out, crossterm::cursor::Hide)?;
        let _ = crossterm::execute!(
            out,
            crossterm::terminal::SetTitle("NotToNotice(); - CRYMACHINA terminal MV")
        );
        out.flush()?;
        Ok(Term { out })
    }

    pub fn write_frame(&mut self, frame: &str) {
        let _ = self.out.write_all(frame.as_bytes());
        let _ = self.out.flush();
    }

    pub fn restore(&mut self) {
        let mut out = std::io::stdout();
        let _ = crossterm::execute!(out, crossterm::cursor::Show);
        let _ = crossterm::execute!(out, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

#[derive(Debug)]
pub enum Input {
    Quit,
    TogglePause,
    Seek(f64),
    VolUp,
    VolDown,
    ToggleFps,
    Help,
    Any,
    Resize,
    None_,
}

pub fn poll_input(timeout: std::time::Duration) -> Input {
    if crossterm::event::poll(timeout).unwrap_or(false) {
        match crossterm::event::read() {
            Ok(Event::Key(KeyEvent {
                code,
                modifiers,
                kind: crossterm::event::KeyEventKind::Press,
                ..
            })) => match code {
                KeyCode::Char('q') | KeyCode::Esc => Input::Quit,
                KeyCode::Char('Q') => Input::Quit,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Input::Quit,
                KeyCode::Char(' ') => Input::TogglePause,
                KeyCode::Left => Input::Seek(-5.0),
                KeyCode::Right => Input::Seek(5.0),
                KeyCode::Char('-') | KeyCode::Char('_') => Input::VolDown,
                KeyCode::Char('=') | KeyCode::Char('+') => Input::VolUp,
                KeyCode::Char('f') => Input::ToggleFps,
                KeyCode::Char('h') | KeyCode::Char('?') => Input::Help,
                _ => Input::Any,
            },
            Ok(Event::Resize(_, _)) => Input::Resize,
            _ => Input::Any,
        }
    } else {
        Input::None_
    }
}
