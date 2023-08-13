use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt;

use crate::{app::app::App, Interrupted};

mod rendering;

const TICK_RATE: Duration = Duration::from_millis(250);

pub(crate) async fn main_loop(
    mut interrupt_rx: broadcast::Receiver<Interrupted>,
    app: Arc<RwLock<App>>,
) -> anyhow::Result<Interrupted> {
    let mut terminal = setup_terminal()?;
    let mut ticker = tokio::time::interval(TICK_RATE);
    let mut crossterm_events = EventStream::new();

    let result: anyhow::Result<Interrupted> = loop {
        tokio::select! {
            // Tick to terminate the select every N milliseconds
            _ = ticker.tick() => (),
            // Catch and handle crossterm events
           maybe_event = crossterm_events.next() => match maybe_event {
                Some(Ok(Event::Key(key)))  => {
                    let mut app = app.write().await;

                    app.handle_key_event(key).await;
                },
                None => break Ok(Interrupted::UserInt),
                _ => (),
            },
            // Catch and handle interrupt signal to gracefully shutdown
            Ok(interrupted) = interrupt_rx.recv() => {
                break Ok(interrupted);
            }
        }

        {
            let app = app.read().await;

            if let Err(err) = terminal
                .draw(|frame| rendering::render_app_too_frame(frame, &app))
                .context("could not render to the terminal")
            {
                break Err(err);
            }
        }
    };

    restore_terminal(&mut terminal)?;

    Ok(result?)
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();

    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(terminal.show_cursor()?)
}
