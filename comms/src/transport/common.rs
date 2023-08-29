use std::pin::Pin;

use tokio_stream::Stream;

pub const NEW_LINE: &[u8; 2] = b"\r\n";

pub type BoxedStream<Item> = Pin<Box<dyn Stream<Item = Item> + Send>>;
