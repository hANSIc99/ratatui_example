#![allow(unused_imports)]
use crossterm::{
    execute,
    event::{self, KeyCode::Char, KeyEventKind, Event::Key},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::io::{stdout};
use anyhow::Result;
pub type Frame<'a> = ratatui::Frame<'a>;

// https://ratatui.rs/tutorial/counter-app/tui.html

/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Application updater.
pub mod update;

struct App {
    counter: i64,
    should_quit: bool,
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn ui(app: &App, f: &mut Frame<'_>) {
    f.render_widget(Paragraph::new(format!("Counter: {}", app.counter)), f.size());
}

fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
      if let Key(key) = event::read()? {
        if key.kind == event::KeyEventKind::Press {
          match key.code {
            Char('j') => app.counter += 1,
            Char('k') => app.counter -= 1,
            Char('q') => app.should_quit = true,
            _ => {},
          }
        }
      }
    }
    Ok(())
}

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
  
    // application state
    let mut app = App { counter: 0, should_quit: false };
  
    loop {
      // application render
      t.draw(|f| {
        ui(&app, f);
      })?;
  
      // application update
      update(&mut app)?;
  
      // application exit
      if app.should_quit {
        break;
      }
    }
  
    Ok(())
}
fn main() -> Result<()> {
    startup()?;
    let status = run();
    shutdown()?;
    status?;
    Ok(())
}
