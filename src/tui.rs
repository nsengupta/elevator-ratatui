use crate::app::{App, AppResult};
use crate::async_event::AppOwnEvent;
use crate::tui_layout::{self, TuiLayout};
use crate::ui;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use futures::{FutureExt, StreamExt};
use ratatui::prelude::Backend;
use ratatui::Terminal;
use std::io;
use std::panic;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,

    /// Layout on screen for various widgets
    pub layout: TuiLayout,

    pub task: JoinHandle<()>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<AppOwnEvent>,
    pub event_tx: UnboundedSender<AppOwnEvent>,
    pub frame_rate: f64,
    pub tick_rate: f64,
}

impl<B: Backend> Tui<B> {

    pub fn new(terminal: Terminal<B>, tui_layout: TuiLayout) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});
        Self {
            terminal,
            layout: tui_layout,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate: 30.0,
            tick_rate: 1.0,
        }
    }

    pub fn start(&mut self) -> AppResult<()> {

        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let frame_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);

        self.cancellation_token.cancel();
        self.cancellation_token = CancellationToken::new();

        let sharable_tx = self.event_tx.clone();
        let sharable_cancellation_token = self.cancellation_token.clone();

        

        self.task = tokio::spawn(async move {

            let mut event_reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut frame_interval = tokio::time::interval(frame_delay);

            // let new_tick_rate = std::time::Duration::from_millis(250);
            // let mut new_interval = tokio::time::interval(tick_rate);

            sharable_tx.send(AppOwnEvent::Init).unwrap();

            loop {

                let next_tick_at = tick_interval.tick();
                let next_frame_at = frame_interval.tick();
                let crossterm_event = event_reader.next().fuse();

                tokio::select! {
                    _ = sharable_cancellation_token.cancelled() => {
                        break;
                      },

                    maybe_event = crossterm_event => {
                        match maybe_event {
                                Some(Ok(evt)) => {
                                    match evt {
                                        CrosstermEvent::Key(key) => {
                                            if key.kind == KeyEventKind::Press {
                                                sharable_tx.send(AppOwnEvent::Key(key)).unwrap();
                                            }
                                        },
                                        CrosstermEvent::Mouse(mouse) => {
                                            sharable_tx.send(AppOwnEvent::Mouse(mouse)).unwrap();
                                        },
                                        _ => {}
                                    }
                            },
                            Some(Err(_)) => {
                                sharable_tx.send(AppOwnEvent::Error).unwrap();
                            },
                            None => {},
                        }
                    },
                    _ = next_tick_at => {
                        sharable_tx.send(AppOwnEvent::Tick).unwrap();
                    },
                    _ = next_frame_at => {
                        sharable_tx.send(AppOwnEvent::Render).unwrap();
                    },
                }
                      
            }

        });


        Ok(())
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> AppResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self, app: &mut App) -> AppResult<()> {
        self.terminal
            .draw(|frame| ui::render_working(app, &self.layout, frame))?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> AppResult<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn stop(&self) -> AppResult<()> {
        self.cancel();
        while !self.task.is_finished() {
          let _ = tokio::time::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> AppResult<()> {
        self.stop()?;
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub async fn next(&mut self) -> Option<AppOwnEvent> {
        self.event_rx.recv().await
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }  
}
