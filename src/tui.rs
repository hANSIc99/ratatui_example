use crossterm::event::{KeyEvent, MouseEvent, EventStream};
use crossterm::event::{Event as CrosstermEvent};
use futures::{StreamExt, FutureExt};
use tokio::{sync::mpsc::{self, UnboundedReceiver, UnboundedSender}, task::JoinHandle};
use anyhow::Result;
use ratatui::{
  prelude::{CrosstermBackend, Terminal},
  widgets::Paragraph,
};

#[derive(Clone, Debug)]
pub enum Event {
  Init,
  Quit,
  Error,
  Closed,
  Tick,
  Render,
  FocusGained,
  FocusLost,
  Paste(String),
  Key(KeyEvent),
  Mouse(MouseEvent),
  Resize(u16, u16),
}

// #[derive(Clone, Copy, Debug)]
// pub enum Event {
//   Error,
//   Tick,
//   Key(KeyEvent),
// }

#[derive(Debug)]
pub struct EventHandler {
  _tx: UnboundedSender<Event>,
  rx: UnboundedReceiver<Event>,
  task: Option<JoinHandle<()>>,
}

impl EventHandler {
  pub fn new() -> Self {
    let tick_rate = std::time::Duration::from_millis(250);

    let (tx, rx) = mpsc::unbounded_channel();
    let _tx = tx.clone();

    let task = tokio::spawn(async move {
      let mut reader = crossterm::event::EventStream::new();
      let mut interval = tokio::time::interval(tick_rate);
      loop {
        let delay = interval.tick();
        let crossterm_event = reader.next().fuse();
        tokio::select! {
          maybe_event = crossterm_event => {
            match maybe_event {
              Some(Ok(evt)) => {
                match evt {
                  crossterm::event::Event::Key(key) => {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                      tx.send(Event::Key(key)).unwrap();
                    }
                  },
                  _ => {},
                }
              }
              Some(Err(_)) => {
                tx.send(Event::Error).unwrap();
              }
              None => {},
            }
          },
          _ = delay => {
              tx.send(Event::Tick).unwrap();
          },
        }
      }
    });

    Self { _tx, rx, task: Some(task) }
  }

  pub async fn next(&mut self) -> Result<Event> {
    self.rx.recv().await.ok_or(color_eyre::eyre::eyre!("Unable to get event"))
  }
}

pub struct Tui {
  pub terminal: ratatui::Terminal<CrosstermBackend<std::io::Stderr>>,
  pub task: JoinHandle<()>,
  pub event_rx: UnboundedReceiver<Event>,
  pub event_tx: UnboundedSender<Event>,
  pub frame_rate: f64,
  pub tick_rate: f64,
}

impl Tui {
  pub fn start(&mut self) {
    let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
    let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
    let _event_tx = self.event_tx.clone();
    self.task = tokio::spawn(async move {
      let mut reader = crossterm::event::EventStream::new();
      let mut tick_interval = tokio::time::interval(tick_delay);
      let mut render_interval = tokio::time::interval(render_delay);
      loop {
        let tick_delay = tick_interval.tick();
        let render_delay = render_interval.tick();
        let crossterm_event = reader.next().fuse();
        tokio::select! {
          maybe_event = crossterm_event => {
            match maybe_event {
              Some(Ok(evt)) => {
                match evt {
                  CrosstermEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                      _event_tx.send(Event::Key(key)).unwrap();
                    }
                  },
                }
              }
              Some(Err(_)) => {
                _event_tx.send(Event::Error).unwrap();
              }
              None => {},
            }
          },
          _ = tick_delay => {
              _event_tx.send(Event::Tick).unwrap();
          },
          _ = render_delay => {
              _event_tx.send(Event::Render).unwrap();
          },
        }
      }
    });
  }
      /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
      Self { terminal, events }
    }
  
    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn enter(&mut self) -> Result<()> {
      terminal::enable_raw_mode()?;
      crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
  
      // Define a custom panic hook to reset the terminal properties.
      // This way, you won't have your terminal messed up if an unexpected error happens.
    //   let panic_hook = panic::take_hook();
    //   panic::set_hook(Box::new(move |panic| {
    //     Self::reset().expect("failed to reset the terminal");
    //     panic_hook(panic);
    //   }));
  
      self.terminal.hide_cursor()?;
      self.terminal.clear()?;
      Ok(())
    }
    
    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }
    
    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
    
    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
    self.terminal.draw(|frame| ui::render(app, frame))?;
    Ok(())
    }
}