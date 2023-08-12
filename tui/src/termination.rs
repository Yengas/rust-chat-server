use tokio::{signal::unix::signal, sync::broadcast};

#[derive(Debug, Clone)]
pub enum Interrupted {
    OsSigInt,
    UserInt,
    ServerDisconnected,
}

#[derive(Debug, Clone)]
pub struct Terminator {
    interrupt_tx: broadcast::Sender<Interrupted>,
}

impl Terminator {
    pub fn new(interrupt_tx: broadcast::Sender<Interrupted>) -> Self {
        Self { interrupt_tx }
    }

    pub fn terminate(&mut self, interrupted: Interrupted) -> anyhow::Result<()> {
        self.interrupt_tx.send(interrupted)?;

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
