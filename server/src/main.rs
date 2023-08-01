use anyhow::Context;
use comms::{
    command::UserCommand,
    event::{Event, RoomParticipationEvent},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::WriteHalf, TcpListener},
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    task::JoinSet,
};
use tokio_stream::{wrappers::LinesStream, StreamExt};

const PORT: u16 = 8080;

struct EventWriter<'a> {
    writer: WriteHalf<'a>,
}

impl<'a> EventWriter<'a> {
    fn new(writer: WriteHalf<'a>) -> Self {
        Self { writer }
    }

    async fn write_event(&mut self, event: &Event) -> anyhow::Result<()> {
        let serialized = serde_json::to_string(&event).unwrap();

        self.writer
            .write_all(serialized.as_bytes())
            .await
            .context("failed to write event to socket")?;

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let mut join_set: JoinSet<()> = JoinSet::new();

    let mut interrupt =
        signal(SignalKind::interrupt()).expect("failed to create interrupt signal stream");
    let server = TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .expect("could not bind to the port");
    let (quit_tx, _) = broadcast::channel::<()>(1);

    println!("Listening on port {}", PORT);
    loop {
        tokio::select! {
            _ = interrupt.recv() => {
                println!("Server interrupted. Gracefully shutting down.");
                quit_tx.send(()).expect("failed to send quit signal");
                break;
            }
            Ok((mut socket, _)) = server.accept() => {
                let mut quit_rx = quit_tx.subscribe();

                join_set.spawn(async move {
                    let (reader, writer) = socket.split();
                    let mut lines = LinesStream::new(BufReader::new(reader).lines()).map(|line| {
                        line.map(|line| {
                            serde_json::from_str::<UserCommand>(&line)
                                .expect("failed to deserialize user command from client")
                        })
                    });
                    let mut event_writer = EventWriter::new(writer);

                    loop {
                        tokio::select! {
                            cmd = lines.next() => {
                                if cmd.is_none() {
                                    println!("Client disconnected.");
                                    break;
                                }

                                println!("Received command: {:?}", cmd);

                                event_writer.write_event(&Event::RoomParticipation(RoomParticipationEvent{
                                    room: "test".to_string(),
                                    username: "test".to_string(),
                                    status: comms::event::RoomParticipationStatus::Joined,
                                })).await.expect("failed to write event to socket");
                            }
                            Ok(_) = quit_rx.recv() => {
                                socket.shutdown().await.expect("failed to shutdown socket");
                                println!("Gracefully shutting down user socket.");
                                break;
                            }
                        }
                    }
                });
            }
        }
    }

    while join_set.join_next().await.is_some() {}
    println!("Server shut down");
}
