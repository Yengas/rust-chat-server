use anyhow::Context;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpStream},
};
use tokio_stream::{wrappers::LinesStream, StreamExt};

use crate::{command, event};

use super::common::{BoxedStream, NEW_LINE};

pub type CommandStream = BoxedStream<anyhow::Result<command::UserCommand>>;

pub struct EventWriter {
    writer: OwnedWriteHalf,
}

impl EventWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self { writer }
    }

    pub async fn write(&mut self, event: &event::Event) -> anyhow::Result<()> {
        let mut serialized_bytes = serde_json::to_vec(event)?;
        serialized_bytes.extend_from_slice(NEW_LINE);

        self.writer.write_all(serialized_bytes.as_slice()).await?;

        Ok(())
    }
}

/// Splits a TCP stream into a stream of commands and an event writer.
/// The user can only send commands to the server, hence stream of lines is deserialized into `UserCommand`s
/// and the writer is used to send `Event`s to the user that maybe response to the command or a broadcasted event from another user.
pub fn split_tcp_stream(stream: TcpStream) -> (CommandStream, EventWriter) {
    let (reader, writer) = stream.into_split();

    (
        Box::pin(
            LinesStream::new(BufReader::new(reader).lines()).map(|line| {
                line.context("could not read line from the client")
                    .and_then(|line| {
                        serde_json::from_str::<command::UserCommand>(&line)
                            .context("failed to deserialize command from client")
                    })
            }),
        ),
        EventWriter::new(writer),
    )
}
