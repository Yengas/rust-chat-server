use std::pin::Pin;

use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpStream},
};
use tokio_stream::{wrappers::LinesStream, Stream, StreamExt};

const NEW_LINE: &[u8; 2] = b"\r\n";

pub(super) type BoxedStream<Item> = Pin<Box<dyn Stream<Item = Item> + Send>>;

pub(super) struct CommandWriter<W: AsyncWrite + Unpin> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> CommandWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub async fn write(&mut self, command: &comms::command::UserCommand) -> anyhow::Result<()> {
        let mut serialized_bytes = serde_json::to_vec(command)?;
        serialized_bytes.extend_from_slice(NEW_LINE);

        self.writer.write_all(serialized_bytes.as_slice()).await?;

        Ok(())
    }
}

pub(super) fn split_stream(
    stream: TcpStream,
) -> (
    BoxedStream<Result<comms::event::Event, std::io::Error>>,
    CommandWriter<OwnedWriteHalf>,
) {
    let (reader, writer) = stream.into_split();

    (
        Box::pin(
            LinesStream::new(BufReader::new(reader).lines()).map(|line| {
                line.map(|line| {
                    serde_json::from_str::<comms::event::Event>(&line)
                        .expect("failed to deserialize event from server")
                })
            }),
        ),
        CommandWriter::new(writer),
    )
}
