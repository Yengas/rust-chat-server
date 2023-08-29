/// Transport over TCP implementation for a client to be able to interact with the server
#[cfg(feature = "client")]
pub mod client;
#[cfg(any(feature = "client", feature = "server"))]
mod common;
/// Transport over TCP implementation for a server to interact with a single client TCP Stream
#[cfg(feature = "server")]
pub mod server;
