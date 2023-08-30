use comms::{
    command::{self, UserCommand},
    event::{self, Event},
    transport,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;

const PORT: usize = 8081;

#[tokio::test]
async fn assert_server_client_transport() {
    let (server_collected_commands, client_collected_events) =
        tokio::join!(execute_server(), execute_client());

    assert!(server_collected_commands.is_ok());
    assert!(client_collected_events.is_ok());

    assert_eq!(
        server_collected_commands.unwrap(),
        vec![
            UserCommand::JoinRoom(command::JoinRoomCommand {
                room: "room-1".into(),
            }),
            UserCommand::SendMessage(command::SendMessageCommand {
                room: "room-1".into(),
                content: "content-1".into(),
            }),
        ]
    );

    assert_eq!(
        client_collected_events.unwrap(),
        vec![Event::LoginSuccessful(event::LoginSuccessfulReplyEvent {
            user_id: "user-id-1".into(),
            session_id: "session-id-1".into(),
            rooms: Vec::default(),
        }),]
    );
}

async fn execute_server() -> anyhow::Result<Vec<command::UserCommand>> {
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
    // store commands received from the client
    let mut collected_commands = Vec::new();

    // welcome the user with some login successful reply event
    event_writer
        .write(&Event::LoginSuccessful(event::LoginSuccessfulReplyEvent {
            user_id: "user-id-1".into(),
            session_id: "session-id-1".into(),
            rooms: Vec::default(),
        }))
        .await?;

    // listen for commands from the client until the connection is closed
    while let Some(result) = command_stream.next().await {
        match result {
            // client has sent a valid command which we could read and parse
            Ok(command) => collected_commands.push(command),
            // client has sent a command which we could not read or parse
            // could be a bug in the client, malicious client, breaking api changes etc.
            Err(e) => return Err(anyhow::anyhow!("failed to read command: {}", e)),
        }
    }

    Ok(collected_commands)
}

async fn execute_client() -> anyhow::Result<Vec<event::Event>> {
    // create a client connection to the server
    let tcp_stream = match TcpStream::connect(format!("localhost:{}", PORT)).await {
        Ok(tcp_stream) => tcp_stream,
        Err(e) => return Err(anyhow::anyhow!("failed to connect to server: {}", e)),
    };

    // break the server connection into higher level API for ease of use
    let (mut event_stream, mut command_writer) = transport::client::split_tcp_stream(tcp_stream);
    // store events received from the server
    let mut collected_events = Vec::new();

    // read the welcome event from the server
    match event_stream.next().await {
        // server has sent a valid event which we could read and parse
        Some(Ok(event)) => collected_events.push(event),
        // server has sent an event which we could not read or parse
        // could be a bug in the server, malicious server, breaking api changes etc.
        Some(Err(e)) => return Err(anyhow::anyhow!("could not parse event: {}", e)),
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

    Ok(collected_events)
}
