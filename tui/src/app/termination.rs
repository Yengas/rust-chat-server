use std::sync::{Arc, Mutex};

use tokio::{signal::unix::signal, sync::broadcast};

#[derive(Debug, Clone)]
pub enum Interrupted {
    OsSigInt,
    UserInt,
}

#[derive(Debug, Clone)]
pub struct Terminator {
    interrupt_tx: Arc<Mutex<broadcast::Sender<Interrupted>>>,
}

impl Terminator {
    pub fn new(interrupt_tx: broadcast::Sender<Interrupted>) -> Self {
        Self {
            interrupt_tx: Arc::new(Mutex::new(interrupt_tx)),
        }
    }

    pub fn terminate(&mut self, interrupted: Interrupted) -> anyhow::Result<()> {
        let interrupt_tx = self.interrupt_tx.lock().unwrap();

        interrupt_tx.send(interrupted)?;

        Ok(())
    }
}

// create a broadcast channel for retrieving the application kill signal
pub fn create_termination() -> (Terminator, broadcast::Receiver<Interrupted>) {
    let mut interrupt_signal = signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("failed to create interrupt signal stream");
    let (tx, rx) = broadcast::channel(1);
    let terminator = Terminator::new(tx);

    tokio::spawn({
        let mut terminator = terminator.clone();

        async move {
            interrupt_signal.recv().await;

            terminator
                .terminate(Interrupted::OsSigInt)
                .expect("failed to send interrupt signal");
        }
    });

    (terminator, rx)
}
