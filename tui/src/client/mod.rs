use std::sync::Arc;

use comms::{command, event};
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc, oneshot},
    task::JoinSet,
};
use tokio_stream::StreamExt;

mod raw_client;

#[derive(Debug, Clone)]
pub struct Client {
    _join_set: Arc<JoinSet<()>>,
    event_broadcast_rx: Arc<broadcast::Receiver<event::Event>>,
    command_mpsc_tx: mpsc::Sender<(oneshot::Sender<anyhow::Result<()>>, command::UserCommand)>,
}

impl Client {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(url).await?;
        let (mut event_stream, mut command_writer) = raw_client::split_stream(stream);

        let mut join_set = JoinSet::new();
        let (event_broadcast_tx, event_broadcast_rx) = broadcast::channel(100);
        let (command_mpsc_tx, mut command_mpsc_rx) =
            mpsc::channel::<(oneshot::Sender<anyhow::Result<()>>, command::UserCommand)>(100);

        join_set.spawn(async move {
            while let Some(Ok(event)) = event_stream.as_mut().next().await {
                let _ = event_broadcast_tx.send(event);
            }
        });

        join_set.spawn({
            async move {
                while let Some((tx, command)) = command_mpsc_rx.recv().await {
                    let result = command_writer.write(&command).await;
                    let _ = tx.send(result);
                }
            }
        });

        Ok(Self {
            _join_set: Arc::new(join_set),
            event_broadcast_rx: Arc::new(event_broadcast_rx),
            command_mpsc_tx,
        })
    }

    pub async fn send_command(&mut self, command: &command::UserCommand) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.command_mpsc_tx.send((tx, command.clone())).await?;

        rx.await?
    }

    pub fn event_stream(&mut self) -> broadcast::Receiver<event::Event> {
        self.event_broadcast_rx.resubscribe()
    }
}
