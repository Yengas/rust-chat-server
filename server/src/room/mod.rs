use comms::event;
use tokio::sync::broadcast;

pub use self::message_sender::MessageSender;
use self::room_participation_list::RoomParticipationList;

mod message_sender;
mod room_participation_list;

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
    broadcast_tx: broadcast::Sender<event::Event>,
    room_participation_list: RoomParticipationList,
}

impl ChatRoom {
    pub fn new(metadata: &ChatRoomMetadata) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);

        ChatRoom {
            name: String::from(&metadata.name),
            broadcast_tx,
            room_participation_list: RoomParticipationList::new(),
        }
    }

    pub fn participants(&self) -> Vec<String> {
        self.room_participation_list.get_unique_usernames()
    }

    /// Add a participant to the room and broadcast that they joined
    /// A UserRoomParticipation is returned for the user to be able to interact with the room
    pub fn add_participant(
        &mut self,
        session_id: String,
        username: String,
    ) -> UserRoomParticipation {
        let broadcast_tx = self.broadcast_tx.clone();
        let broadcast_rx = broadcast_tx.subscribe();
        let message_sender = MessageSender::new(
            broadcast_tx,
            self.name.clone(),
            session_id,
            username.clone(),
        );

        // If the user is new e.g. they do not have another session with same username,
        // broadcast that they joined to all users
        if self.room_participation_list.insert_user(&message_sender) {
            let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
                event::RoomParticipationBroacastEvent {
                    username,
                    room: self.name.clone(),
                    status: event::RoomParticipationStatus::Joined,
                },
            ));
        }

        UserRoomParticipation::new(message_sender, broadcast_rx)
    }

    /// Remove a participant from the room and broadcast that they left
    /// Consume the MessageSender to drop it
    pub fn remove_participant(&mut self, message_sender: MessageSender) {
        if self
            .room_participation_list
            .remove_user_by_session(&message_sender)
        {
            let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
                event::RoomParticipationBroacastEvent {
                    username: String::from(message_sender.username()),
                    room: self.name.clone(),
                    status: event::RoomParticipationStatus::Left,
                },
            ));
        }
    }
}
