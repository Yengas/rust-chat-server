use app::{termination::create_termination, App};
use std::sync::Arc;
use tokio::sync::RwLock;

mod app;
mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (terminator, interrupt_rx) = create_termination();
    let app = Arc::new(RwLock::new(App::new(terminator.clone())));

    tokio::try_join!(
        cli::main_loop(interrupt_rx.resubscribe(), app.clone()),
        app::main_loop(interrupt_rx.resubscribe(), app),
    )?;

    Ok(())
}
