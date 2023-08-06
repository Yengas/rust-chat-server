use std::sync::Arc;

use comms::{
    command::UserCommand,
    event::{Event, UserMessageEvent},
};
use tokio::{
    net::TcpStream,
    sync::{broadcast, Mutex},
};
use tokio_stream::StreamExt;

mod raw_socket;

struct MessageSender {
    broacast_tx: broadcast::Sender<Event>,
    room: String,
    username: String,
}

impl MessageSender {
    fn new(broadcast_tx: broadcast::Sender<Event>, room: String, username: String) -> Self {
        MessageSender {
            broacast_tx: broadcast_tx,
            room,
            username,
        }
    }

    fn send(&self, content: String) -> anyhow::Result<()> {
        self.broacast_tx
            .send(comms::event::Event::UserMessage(UserMessageEvent {
                room: self.room.clone(),
                username: self.username.clone(),
                content,
            }))?;

        Ok(())
    }
}

pub struct ChatRoom {
    name: String,
    participants: Vec<String>,
    broadcast_tx: broadcast::Sender<Event>,
}

impl ChatRoom {
    pub fn new(name: String) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);

        ChatRoom {
            name,
            broadcast_tx,
            participants: Vec::new(),
        }
    }

    fn add_participant(&mut self, username: String) -> (MessageSender, broadcast::Receiver<Event>) {
        self.participants.push(username.clone());

        let broadcast_tx = self.broadcast_tx.clone();
        let broadcast_rx = broadcast_tx.subscribe();
        let message_sender = MessageSender::new(broadcast_tx, self.name.clone(), username);

        (message_sender, broadcast_rx)
    }
}

pub async fn handle_user_session(
    chat_room: Arc<Mutex<ChatRoom>>,
    mut quit_rx: broadcast::Receiver<()>,
    stream: TcpStream,
) -> anyhow::Result<()> {
    let username = nanoid::nanoid!();
    let (mut commands, mut event_writer) = raw_socket::split_stream(stream);
    let (message_sender, mut broadcast_rx) = chat_room.lock().await.add_participant(username);

    loop {
        tokio::select! {
            cmd = commands.next() => {
                if cmd.is_none() {
                    println!("Client disconnected.");
                    break;
                }

                match cmd.unwrap().unwrap() {
                    UserCommand::Quit(_) => {
                        println!("Client quit.");
                        break;
                    }
                    UserCommand::SendMessage(cmd) => {
                        message_sender.send(cmd.content)?;
                    }
                    _ => {}
                }
            }
            Ok(event) = broadcast_rx.recv() => {
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
