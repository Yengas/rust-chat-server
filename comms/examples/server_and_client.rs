use anyhow::Context;
use comms::{
    command::{self, UserCommand},
    event::{self, Event},
    transport,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;

const PORT: usize = 8081;

async fn server_example() -> anyhow::Result<()> {
    // bind to the example port to wait for client connection
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .expect("could not bind to the port");

    // accept the only client connection we will have
    let tcp_stream = match listener.accept().await {
        Ok((tcp_stream, _addr)) => tcp_stream,
        Err(e) => return Err(anyhow::anyhow!("failed to accept client: {}", e)),
    };

    // break the client connection into higher level API for ease of use
    let (mut command_stream, mut event_writer) = transport::server::split_tcp_stream(tcp_stream);

    // welcome the user with some login successful reply event
    event_writer
        .write(&Event::LoginSuccessful(event::LoginSuccessfulReplyEvent {
            username: "username-1".into(),
            session_id: "session-id-1".into(),
            rooms: Vec::default(),
        }))
        .await?;

    // listen for commands from the client until the connection is closed
    while let Some(result) = command_stream.next().await {
        match result {
            // client has sent a valid command which we could read and parse
            Ok(command) => println!("SERVER: received command: {:?}", command),
            // client has sent a command which we could not read or parse
            // could be a bug in the client, malicious client, breaking api changes etc.
            Err(e) => println!("SERVER: failed to read command: {}", e),
        }
    }

    Ok(())
}

async fn client_example() -> anyhow::Result<()> {
    // create a client connection to the server
    let tcp_stream = match TcpStream::connect(format!("localhost:{}", PORT)).await {
        Ok(tcp_stream) => tcp_stream,
        Err(e) => return Err(anyhow::anyhow!("failed to connect to server: {}", e)),
    };

    // break the server connection into higher level API for ease of use
    let (mut event_stream, mut command_writer) = transport::client::split_tcp_stream(tcp_stream);

    // read the welcome event from the server
    match event_stream.next().await {
        // server has sent a valid event which we could read and parse
        Some(Ok(event)) => println!("CLIENT: received event: {:?}", event),
        // server has sent an event which we could not read or parse
        // could be a bug in the server, malicious server, breaking api changes etc.
        Some(Err(e)) => println!("CLIENT: failed to read event: {}", e),
        // server has closed the connection, return an error
        None => return Err(anyhow::anyhow!("server closed the connection")),
    }

    // send some commands to the server
    command_writer
        .write(&UserCommand::JoinRoom(command::JoinRoomCommand {
            room: "room-1".into(),
        }))
        .await?;

    command_writer
        .write(&UserCommand::SendMessage(command::SendMessageCommand {
            room: "room-1".into(),
            content: "content-1".into(),
        }))
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::try_join!(server_example(), client_example()).context("one of the examples failed")?;

    println!("example ran without problems");

    Ok(())
}
