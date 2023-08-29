use anyhow::Context;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpStream},
};
use tokio_stream::{wrappers::LinesStream, StreamExt};

use crate::{command, event};

use super::common::{BoxedStream, NEW_LINE};

/// [EventStream] is a stream of [crate::event::Event]s sent by the server
///
/// # Cancel Safety
///
/// This stream is cancel-safe, meaning that it can be used in [tokio::select]
/// without the risk of missing events.
pub type EventStream = BoxedStream<anyhow::Result<event::Event>>;

/// [CommandWriter] is a wrapper around a [TcpStream] which writes [crate::command::UserCommand]s to the server
pub struct CommandWriter {
    writer: OwnedWriteHalf,
}

impl CommandWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self { writer }
    }

    /// Send a [crate::command::UserCommand] to the backing [TcpStream]
    ///
    /// # Cancel Safety
    ///
    /// This method is not cancellation safe. If it is used as the event
    /// in a [tokio::select!] statement and some other
    /// branch completes first, then the provided [crate::command::UserCommand] may have been
    /// partially written, but future calls to `write` will start over
    /// from the beginning of the buffer. Causing undefined behaviour.
    pub async fn write(&mut self, command: &command::UserCommand) -> anyhow::Result<()> {
        let mut serialized_bytes = serde_json::to_vec(command)?;
        serialized_bytes.extend_from_slice(NEW_LINE);

        self.writer.write_all(serialized_bytes.as_slice()).await?;

        Ok(())
    }
}

/// Splits a TCP stream into a stream of events and a command writer.
///
/// # Arguments
///
/// - `stream` - A [TcpStream] to split
pub fn split_tcp_stream(stream: TcpStream) -> (EventStream, CommandWriter) {
    let (reader, writer) = stream.into_split();

    (
        Box::pin(
            LinesStream::new(BufReader::new(reader).lines()).map(|line| {
                line.context("could not read line from the server")
                    .and_then(|line| {
                        serde_json::from_str::<event::Event>(&line)
                            .context("failed to deserialize event from the server")
                    })
            }),
        ),
        CommandWriter::new(writer),
    )
}
