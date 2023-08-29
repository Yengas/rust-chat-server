use anyhow::Context;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpStream},
};
use tokio_stream::{wrappers::LinesStream, StreamExt};

use crate::{command, event};

use super::common::{BoxedStream, NEW_LINE};

/// [CommandStream] is a stream of [crate::command::UserCommand]s sent by the client
///
/// # Cancel Safety
///
/// This stream is cancel-safe, meaning that it can be used in [tokio::select!]
/// without the risk of missing commands.
pub type CommandStream = BoxedStream<anyhow::Result<command::UserCommand>>;

/// [EventWriter] is a wrapper around a [TcpStream] which writes [crate::event::Event]s to the client
pub struct EventWriter {
    writer: OwnedWriteHalf,
}

impl EventWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self { writer }
    }

    /// Send a [crate::event::Event] to the backing [TcpStream]
    ///
    /// # Cancel Safety
    ///
    /// This method is not cancellation safe. If it is used as the event
    /// in a [tokio::select!] statement and some other
    /// branch completes first, then the provided [crate::event::Event] may have been
    /// partially written, but future calls to `write` will start over
    /// from the beginning of the buffer. Causing undefined behaviour.
    pub async fn write(&mut self, event: &event::Event) -> anyhow::Result<()> {
        let mut serialized_bytes = serde_json::to_vec(event)?;
        serialized_bytes.extend_from_slice(NEW_LINE);

        self.writer.write_all(serialized_bytes.as_slice()).await?;

        Ok(())
    }
}

/// Splits a TCP stream into a stream of commands and an event writer.
///
/// # Arguments
///
/// - `stream` - A [TcpStream] to split
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
