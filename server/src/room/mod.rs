use std::collections::HashSet;

use anyhow::Context;
use comms::event;
use tokio::sync::broadcast;

#[derive(Debug)]
/// MessageSender is a struct that allows sending messages associated to a specific room
/// and a username. When a user joins a room, a MessageSender is created for that user.
pub struct MessageSender {
    broacast_tx: broadcast::Sender<event::Event>,
    room: String,
    username: String,
}

impl MessageSender {
    fn new(broadcast_tx: broadcast::Sender<event::Event>, room: String, username: String) -> Self {
        MessageSender {
            broacast_tx: broadcast_tx,
            room,
            username,
        }
    }

    /// Send a message to the room
    pub fn send(&self, content: String) -> anyhow::Result<()> {
        self.broacast_tx
            .send(comms::event::Event::UserMessage(
                event::UserMessageBroadcastEvent {
                    room: self.room.clone(),
                    username: self.username.clone(),
                    content,
                },
            ))
            .context("could not write to the broadcast channel")?;

        Ok(())
    }
}

#[derive(Debug)]
/// UserRoomParticipation is a struct that holds a MessageSender and a broadcast receiver.
/// When a user joins a room, a UserRoomParticipation is handed out to that user.
pub struct UserRoomParticipation {
    pub message_sender: MessageSender,
    pub broadcast_rx: broadcast::Receiver<event::Event>,
}

impl UserRoomParticipation {
    fn new(message_sender: MessageSender, broadcast_rx: broadcast::Receiver<event::Event>) -> Self {
        UserRoomParticipation {
            message_sender,
            broadcast_rx,
        }
    }
}

#[derive(Debug, Clone)]
/// ChatRoomMetadata is a struct that holds the metadata of a chat room.
pub struct ChatRoomMetadata {
    pub name: String,
    pub description: String,
}

impl ChatRoomMetadata {
    pub fn new(name: &str, description: &str) -> Self {
        ChatRoomMetadata {
            name: String::from(name),
            description: String::from(description),
        }
    }
}

#[derive(Debug)]
/// ChatRoom is a struct that handles the participants of a chat room and the primary broadcast channel
/// A UserRoomParticipation is handed out to a user when they join a room
pub struct ChatRoom {
    name: String,
    participants: HashSet<String>,
    broadcast_tx: broadcast::Sender<event::Event>,
}

impl ChatRoom {
    pub fn new(metadata: &ChatRoomMetadata) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);

        ChatRoom {
            name: String::from(&metadata.name),
            broadcast_tx,
            participants: HashSet::new(),
        }
    }

    pub fn participants(&self) -> &HashSet<String> {
        &self.participants
    }

    /// Add a participant to the room and broadcast that they joined
    /// A UserRoomParticipation is returned for the user to be able to interact with the room
    pub fn add_participant(&mut self, username: String) -> UserRoomParticipation {
        self.participants.insert(username.clone());

        let broadcast_tx = self.broadcast_tx.clone();
        let broadcast_rx = broadcast_tx.subscribe();
        let message_sender = MessageSender::new(broadcast_tx, self.name.clone(), username.clone());

        let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
            event::RoomParticipationBroacastEvent {
                username,
                room: self.name.clone(),
                status: event::RoomParticipationStatus::Joined,
            },
        ));

        UserRoomParticipation::new(message_sender, broadcast_rx)
    }

    /// Remove a participant from the room and broadcast that they left
    pub fn remove_participant(&mut self, username: &String) {
        self.participants.retain(|p| p != username);

        let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
            event::RoomParticipationBroacastEvent {
                username: username.clone(),
                room: self.name.clone(),
                status: event::RoomParticipationStatus::Left,
            },
        ));
    }
}
