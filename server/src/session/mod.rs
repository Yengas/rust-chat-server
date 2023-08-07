use std::{collections::HashMap, sync::Arc};

use comms::command::UserCommand;
use tokio::{
    net::TcpStream,
    sync::{broadcast, Mutex},
};
use tokio_stream::StreamExt;

use crate::room::ChatRoom;

use self::room_manager::ChatRoomManager;

mod raw_socket;
mod room_manager;

pub async fn handle_user_session(
    chat_rooms: HashMap<String, Arc<Mutex<ChatRoom>>>,
    mut quit_rx: broadcast::Receiver<()>,
    stream: TcpStream,
) -> anyhow::Result<()> {
    let username = nanoid::nanoid!();
    let (mut commands, mut event_writer) = raw_socket::split_stream(stream);
    let mut room_manager = ChatRoomManager::new(&username, chat_rooms);

    loop {
        tokio::select! {
            cmd = commands.next() => match cmd {
                None | Some(Ok(UserCommand::Quit(_))) => {
                    println!("Client quit.");
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
