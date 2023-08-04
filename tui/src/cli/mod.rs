use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};

use crate::state::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt;

mod ui;

pub async fn main_loop(
    interrupt_rx: broadcast::Receiver<()>,
    app: Arc<RwLock<App>>,
) -> anyhow::Result<()> {
    let mut terminal = setup_terminal()?;

    run(&mut terminal, interrupt_rx, app).await?;

    restore_terminal(&mut terminal)?;

    Ok(())
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

async fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut interrupt_rx: broadcast::Receiver<()>,
    app: Arc<RwLock<App>>,
) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_millis(250));
    let mut crossterm_events = EventStream::new();

    loop {
        tokio::select! {
            // Tick to terminate the select every N milliseconds
            _ = ticker.tick() => (),
            // Catch and handle crossterm events
            Some(Ok(Event::Key(key))) = crossterm_events.next() => {
                let mut app = app.write().await;
                // TODO: handle the error case properly
                if app.handle_event(key).is_err() {
                    return Ok(());
                }
            }
            // Catch and handle interrupt signal to gracefully shutdown
            Ok(_) = interrupt_rx.recv() => {
                return Ok(());
            }
        }

        let app = app.read().await;
        terminal.draw(|frame| ui::render_app_too_frame(frame, &app))?;
    }
}
