use std::{collections::HashMap, sync::Arc};

use comms::{
    command::UserCommand,
    event::{self, RoomDetail},
    transport,
};
use nanoid::nanoid;
use tokio::{
    net::TcpStream,
    sync::{broadcast, Mutex},
};
use tokio_stream::StreamExt;

use crate::room::{ChatRoom, ChatRoomMetadata};

use self::room_manager::ChatRoomManager;

mod room_manager;

/// Given a tcp stream and a global chat room list, handles the user session
/// until the user quits the session, the tcp stream is closed for some reason, or the server shuts down
pub async fn handle_user_session(
    chat_rooms: Vec<(ChatRoomMetadata, Arc<Mutex<ChatRoom>>)>,
    mut quit_rx: broadcast::Receiver<()>,
    stream: TcpStream,
) -> anyhow::Result<()> {
    let session_id = nanoid!();
    // Generate a random username for the user, since we don't have a login system
    let username = String::from(&nanoid!()[0..5]);
    // Split the tcp stream into a command stream and an event writer with better ergonomics
    let (mut commands, mut event_writer) = transport::server::split_tcp_stream(stream);

    // Welcoming the user with a login successful event and necessary information about the server
    event_writer
        .write(&event::Event::LoginSuccessful(
            event::LoginSuccessfulReplyEvent {
                session_id: session_id.clone(),
                username: username.clone(),
                rooms: chat_rooms
                    .iter()
                    .map(|(metadata, _)| RoomDetail {
                        name: metadata.name.clone(),
                        description: metadata.description.clone(),
                    })
                    .collect(),
            },
        ))
        .await?;

    // Create a chat room manager with the given global chat rooms
    // Room manager will abstract the room management logic from the session handler
    let chat_rooms = chat_rooms
        .into_iter()
        .map(|(metadata, room)| (metadata.name, room))
        .collect::<HashMap<_, _>>();
    let mut room_manager = ChatRoomManager::new(&session_id, &username, chat_rooms);

    loop {
        tokio::select! {
            cmd = commands.next() => match cmd {
                // If the user closes the tcp stream, or sends a quit cmd
                // We need to cleanup resources in a way that the other users are notified about the user's departure
                None | Some(Ok(UserCommand::Quit(_))) => {
                    room_manager.leave_all_rooms().await?;
                    break;
                }
                // Handle a valid user command
                Some(Ok(cmd)) => match cmd {
                    // For room management commands, we need to handle them in the room manager
                    UserCommand::JoinRoom(_) | UserCommand::SendMessage(_) | UserCommand::LeaveRoom(_) => {
                        room_manager.handle_user_command(cmd).await?;
                    }
                    _ => {}
                }
                _ => {}
            },
            // Aggregated events from the room manager are sent to the user
            Ok(event) = room_manager.recv() => {
                event_writer.write(&event).await?;
            }
            // If the server is shutting down, we can just close the tcp streams
            // and exit the session handler. Since the server is shutting down,
            // we don't need to notify other users about the user's departure or cleanup resources
            Ok(_) = quit_rx.recv() => {
                drop(event_writer);
                println!("Gracefully shutting down user tcp stream.");
                break;
            }
        }
    }

    Ok(())
}
