use std::{collections::HashMap, sync::Arc};

use comms::{
    command::UserCommand,
    event::{self, RoomDetail},
};
use tokio::{
    net::TcpStream,
    sync::{broadcast, Mutex},
};
use tokio_stream::StreamExt;

use crate::room::{ChatRoom, ChatRoomMetadata};

use self::room_manager::ChatRoomManager;

mod raw_socket;
mod room_manager;

pub async fn handle_user_session(
    chat_rooms: Vec<(ChatRoomMetadata, Arc<Mutex<ChatRoom>>)>,
    mut quit_rx: broadcast::Receiver<()>,
    stream: TcpStream,
) -> anyhow::Result<()> {
    let username = String::from(&nanoid::nanoid!()[0..5]);
    let (mut commands, mut event_writer) = raw_socket::split_stream(stream);

    event_writer
        .write(&event::Event::LoginSuccessful(
            event::LoginSuccessfulEvent {
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

    let chat_rooms = chat_rooms
        .into_iter()
        .map(|(metadata, room)| (metadata.name, room))
        .collect::<HashMap<_, _>>();
    let mut room_manager = ChatRoomManager::new(&username, chat_rooms);

    loop {
        tokio::select! {
            cmd = commands.next() => match cmd {
                None | Some(Ok(UserCommand::Quit(_))) => {
                    room_manager.leave_all_rooms().await?;
                    break;
                }
                Some(Ok(cmd)) => match cmd {
                    UserCommand::JoinRoom(_) | UserCommand::SendMessage(_) | UserCommand::LeaveRoom(_) => {
                        room_manager.handle_user_command(cmd).await?;
                    }
                    _ => {}
                }
                _ => {}
            },
            Ok(event) = room_manager.recv() => {
                event_writer.write(&event).await?;
            }
            Ok(_) = quit_rx.recv() => {
                drop(event_writer);
                println!("Gracefully shutting down user socket.");
                break;
            }
        }
    }

    Ok(())
}
