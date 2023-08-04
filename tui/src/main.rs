use state::App;
use std::sync::Arc;
use tokio::{
    signal::unix::signal,
    sync::{broadcast, RwLock},
};

mod cli;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let interrupt_rx = create_interrupt_signal_channel();
    let app = Arc::new(RwLock::new(App::default()));

    tokio::try_join!(cli::main_loop(interrupt_rx.resubscribe(), app.clone()))?;

    Ok(())
}

// create a broadcast channel for retrieving the application kill signal
fn create_interrupt_signal_channel() -> broadcast::Receiver<()> {
    let mut interrupt_signal = signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("failed to create interrupt signal stream");
    let (tx, rx) = broadcast::channel(1);

    tokio::spawn(async move {
        interrupt_signal.recv().await;
        println!("received interrupt signal");
        tx.send(()).expect("failed to send interrupt signal");
    });

    rx
}
