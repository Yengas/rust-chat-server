use app::{termination::create_termination, App};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock};

mod app;
mod cli;
mod client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stream = TcpStream::connect("localhost:8080").await?;
    let (event_stream, command_writer) = client::split_stream(stream);
    let (terminator, interrupt_rx) = create_termination();

    let app = Arc::new(RwLock::new(App::new(command_writer, terminator.clone())));

    tokio::try_join!(
        cli::main_loop(interrupt_rx.resubscribe(), Arc::clone(&app)),
        app::main_loop(interrupt_rx.resubscribe(), event_stream, app),
    )?;

    Ok(())
}
