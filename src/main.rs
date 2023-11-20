mod tui;

use std::time::Duration;

//use color_eyre::eyre::Result;
use anyhow::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::{self, UnboundedSender};
use tui::{Event, EventHandler, Tui};
use crossterm::{
  event::{self, Event::Key, KeyCode::Char},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
// App state
struct App {
  counter: i64,
  should_quit: bool,
  action_tx: UnboundedSender<Action>,
}

// App actions
#[derive(Clone)]
pub enum Action {
  Tick,
  Increment,
  Decrement,
  NetworkRequestAndThenIncrement, // new
  NetworkRequestAndThenDecrement, // new
  Quit,
  Render,
  None,
}

// App ui render function
fn ui(f: &mut Frame<'_>, app: &mut App) {
  let area = f.size();
  f.render_widget(
    Paragraph::new(format!("Press j or k to increment or decrement.\n\nCounter: {}", app.counter,))
      .block(
        Block::default()
          .title("ratatui async counter app")
          .title_alignment(Alignment::Center)
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded),
      )
      .style(Style::default().fg(Color::Cyan))
      .alignment(Alignment::Center),
    area,
  );
}

fn get_action(_app: &App, event: Event) -> Action {
  match event {
    Event::Error => Action::None,
    Event::Tick => Action::Tick,
    Event::Render => Action::Render,
    Event::Key(key) => {
      match key.code {
        Char('j') => Action::Increment,
        Char('k') => Action::Decrement,
        Char('J') => Action::NetworkRequestAndThenIncrement, // new
        Char('K') => Action::NetworkRequestAndThenIncrement, // new
        Char('q') => Action::Quit,
        _ => Action::None,
      }
    },
    _ => Action::None,
  }
}

fn update(app: &mut App, action: Action) {
  match action {
    Action::Increment => {
      app.counter += 1;
    },
    Action::Decrement => {
      app.counter -= 1;
    },
    Action::NetworkRequestAndThenIncrement => {
      let tx = app.action_tx.clone();
      tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await; // simulate network request
        tx.send(Action::Increment).unwrap();
      });
    },
    Action::NetworkRequestAndThenDecrement => {
      let tx = app.action_tx.clone();
      tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await; // simulate network request
        tx.send(Action::Decrement).unwrap();
      });
    },
    Action::Quit => app.should_quit = true,
    _ => {},
  };
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

async fn run() -> Result<()> {

  let mut events = tui::EventHandler::new(); // new
  let (action_tx, mut action_rx) = mpsc::unbounded_channel(); // new
  // ratatui terminal
  let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

  // application state
  //let mut app = App { counter: 0, should_quit: false };
  let mut app = App { counter: 0, should_quit: false, action_tx: action_tx.clone() };

  loop {
    let event = events.next().await?; // new

    // // application update
    // update(&mut app, event)?;

    // // application render
    // t.draw(|f| {
    //   ui(f, &app);
    // })?;

    // application exit
    if app.should_quit {
      break;
    }
  }

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  // setup terminal
  startup()?;

  let result = run().await;

  // teardown terminal before unwrapping Result of app run
  shutdown()?;

  result?;

  Ok(())
}

// async fn run() -> Result<()> {
//   let (action_tx, mut action_rx) = mpsc::unbounded_channel(); // new

//   let backend = CrosstermBackend::new(std::io::stderr());
//   let terminal = Terminal::new(backend)?;
//   let events = EventHandler::new();
//   //let mut tui = Tui::new(, );
//   // ratatui terminal
//   let mut tui = tui::Tui::new(terminal, events)?.tick_rate(1.0).frame_rate(30.0);
//   tui.enter()?;

//   // application state
//   let mut app = App { counter: 0, should_quit: false, action_tx: action_tx.clone() };

//   loop {
//     let e = tui.next().await?;
//     match e {
//       tui::Event::Quit => action_tx.send(Action::Quit)?,
//       tui::Event::Tick => action_tx.send(Action::Tick)?,
//       tui::Event::Render => action_tx.send(Action::Render)?,
//       tui::Event::Key(_) => {
//         let action = get_action(&app, e);
//         action_tx.send(action.clone())?;
//       },
//       _ => {},
//     };

//     while let Ok(action) = action_rx.try_recv() {
//       // application update
//       update(&mut app, action.clone());
//       // render only when we receive Action::Render
//       if let Action::Render = action {
//         tui.draw(|f| {
//           ui(f, &mut app);
//         })?;
//       }
//     }

//     // application exit
//     if app.should_quit {
//       break;
//     }
//   }
//   tui.exit()?;

//   Ok(())
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//   let result = run().await;

//   result?;

//   Ok(())
// }