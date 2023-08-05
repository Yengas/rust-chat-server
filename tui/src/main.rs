use app::{termination::create_termination, App};
use client::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

mod app;
mod cli;
mod client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("localhost:8080").await?;
    let (terminator, interrupt_rx) = create_termination();
    let app = Arc::new(RwLock::new(App::new(client.clone(), terminator.clone())));

    tokio::try_join!(
        cli::main_loop(interrupt_rx.resubscribe(), app.clone()),
        app::main_loop(interrupt_rx.resubscribe(), client, app),
    )?;

    Ok(())
}
