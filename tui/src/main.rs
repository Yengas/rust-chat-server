use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use state::App;
use tokio_stream::StreamExt;

mod state;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::default();
    let mut terminal = setup_terminal()?;

    run(&mut terminal, app).await?;

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
    mut app: App,
) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_millis(250));
    let mut crossterm_events = EventStream::new();

    loop {
        tokio::select! {
            // Tick to terminate the select every N milliseconds
            _ = ticker.tick() => (),
            // Catch and handle crossterm events
            Some(Ok(Event::Key(key))) = crossterm_events.next() => {
                // TODO: handle the error case properly
                if let Err(_) = app.handle_event(key) {
                    return Ok(());
                }
            }
        }

        terminal.draw(|frame| ui::render_app_too_frame(frame, &mut app))?;
    }
}
