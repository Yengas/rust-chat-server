# Comms Library

The `comms` library, short for "communications," serves as an auxiliary module for the [rust-chat-server](../) project. It provides definitions and utilities for handling events and commands.

## Features

- Definitions and documentation for [events](./src/event.rs) and [commands](./src/command.rs) utilized by the [rust-chat-server](../).
- TCP transport support for both **events** and **commands**.
  - [`comms::transport::client`](./src/transport/client.rs) assists in splitting a [tokio::net::TcpStream](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) into an **EventStream** and a **CommandWriter**.
  - [`comms::transport::server`](./src/transport/server.rs) enables the partitioning of a [tokio::net::TcpStream](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) into a **CommandStream** and an **EventWriter**.

## Example Usage

Execute the e2e test for client and server with the following command: `cargo test --features="client,server"`

[This e2e test](./tests/e2e_server_and_client_transport.rs) spawns a server and a client. The server accepts one client, sends it an event, and listens for commands until the connection is closed. Conversely, the client receives one event, sends two commands, and then terminates its connection.

Here's a simplified pseudocode version of the [e2e test code](./tests/e2e_server_and_client_transport.rs):

```rust
// full e2e test code: src/tests/e2e_server_and_client_transport.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::try_join!(server_example(), client_example())?;
    // All examples succeed if this line is reached.
    Ok(())
}

async fn server_example() -> anyhow::Result<()> {
    let listener = /* Create a TcpListener */;
    let tcp_stream = /* Accept a single client from `listener` */;
    // Use comms::transport to elevate the TcpStream to a higher-level API.
    let (mut command_stream, mut event_writer) = transport::server::split_tcp_stream(tcp_stream);

    event_writer.write(/* Login Successful Event */).await?;

    // Loop to read and print commands until the client closes the connection.
    while let Some(result) = command_stream.next().await {
        match result {
            Ok(command) => println!("SERVER: Command received: {:?}", command),
            Err(e) => println!("SERVER: Failed to read command: {}", e),
        }
    }

    Ok(())
}

async fn client_example() -> anyhow::Result<()> {
    let tcp_stream = /* Connect to the server */;
    // Use comms::transport to elevate the TcpStream to a higher-level API.
    let (mut event_stream, mut command_writer) = transport::client::split_tcp_stream(tcp_stream);

    // Read and print a single event.
    match event_stream.next().await {
        Some(Ok(event)) => println!("CLIENT: Event received: {:?}", event),
        Some(Err(e)) => println!("CLIENT: Failed to read event: {}", e),
        None => return Err(anyhow::anyhow!("Server closed the connection")),
    }

    command_writer.write(/* Join Room Command */).await?;
    command_writer.write(/* Send Message Command */).await?;

    Ok(())
}
```