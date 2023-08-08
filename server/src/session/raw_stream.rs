use std::pin::Pin;

use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpStream},
};
use tokio_stream::{wrappers::LinesStream, Stream, StreamExt};

const NEW_LINE: &[u8; 2] = b"\r\n";

pub(super) type BoxedStream<Item> = Pin<Box<dyn Stream<Item = Item> + Send>>;

pub(super) struct EventWriter<W: AsyncWrite + Unpin> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> EventWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub async fn write(&mut self, event: &comms::event::Event) -> anyhow::Result<()> {
        let mut serialized_bytes = serde_json::to_vec(event)?;
        serialized_bytes.extend_from_slice(NEW_LINE);

        self.writer.write_all(serialized_bytes.as_slice()).await?;

        Ok(())
    }
}

/// Splits a TCP stream into a stream of lines and a writer.
/// The user can only send commands to the server, hence stream of lines is deserialized into `UserCommand`s
/// and the writer is used to send `Event`s to the user that maybe response to the command or a broadcasted event from another user.
pub(super) fn split_stream(
    stream: TcpStream,
) -> (
    BoxedStream<Result<comms::command::UserCommand, std::io::Error>>,
    EventWriter<OwnedWriteHalf>,
) {
    let (reader, writer) = stream.into_split();

    (
        Box::pin(
            LinesStream::new(BufReader::new(reader).lines()).map(|line| {
                line.map(|line| {
                    serde_json::from_str::<comms::command::UserCommand>(&line)
                        .expect("failed to deserialize command from client")
                })
            }),
        ),
        EventWriter::new(writer),
    )
}
