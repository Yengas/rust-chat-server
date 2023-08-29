/// Set of commands which the server can receive and process
pub mod command;
/// Set of events split into Broadcast and Reply events according to their source
pub mod event;
/// Implementation of event and command transportation over TCP Streams.
/// Requires 'server' or 'client' features to be enabled and will bring in tokio dependency alongside with other dependencies
pub mod transport;
