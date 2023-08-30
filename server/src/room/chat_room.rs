use comms::event::{self, Event};
use tokio::sync::broadcast;

use super::{
    user_registry::UserRegistry, user_session_handle::UserSessionHandle, SessionAndUsername,
};

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

const BROADCAST_CHANNEL_CAPACITY: usize = 100;

#[derive(Debug)]
/// [ChatRoom] handles the participants of a chat room and the primary broadcast channel
/// A [UserSessionHandle] is handed out to a user when they join the room
pub struct ChatRoom {
    name: String,
    broadcast_tx: broadcast::Sender<event::Event>,
    user_registry: UserRegistry,
}

impl ChatRoom {
    pub fn new(metadata: &ChatRoomMetadata) -> Self {
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

        ChatRoom {
            name: String::from(&metadata.name),
            broadcast_tx,
            user_registry: UserRegistry::new(),
        }
    }

    pub fn get_unique_user_ids(&self) -> Vec<String> {
        self.user_registry.get_unique_user_ids()
    }

    /// Add a participant to the room and broadcast that they joined
    ///
    /// # Returns
    ///
    /// - A broadcast receiver for the user to receive messages from the room
    /// - A [UserSessionHandle] for the user to be able to interact with the room
    pub fn join(
        &mut self,
        session_and_username: SessionAndUsername,
    ) -> (broadcast::Receiver<Event>, UserSessionHandle) {
        let broadcast_tx = self.broadcast_tx.clone();
        let broadcast_rx = broadcast_tx.subscribe();
        let user_session_handle = UserSessionHandle::new(
            self.name.clone(),
            broadcast_tx,
            session_and_username.clone(),
        );

        // If the user is new e.g. they do not have another session with same username,
        // broadcast that they joined to all users
        if self.user_registry.insert(&user_session_handle) {
            let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
                event::RoomParticipationBroacastEvent {
                    username: session_and_username.username.clone(),
                    room: self.name.clone(),
                    status: event::RoomParticipationStatus::Joined,
                },
            ));
        }

        (broadcast_rx, user_session_handle)
    }

    /// Remove a participant from the room and broadcast that they left
    /// Consume the [UserSessionHandle] to drop it
    pub fn leave(&mut self, user_session_handle: UserSessionHandle) {
        if self.user_registry.remove(&user_session_handle) {
            let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
                event::RoomParticipationBroacastEvent {
                    username: String::from(user_session_handle.username()),
                    room: self.name.clone(),
                    status: event::RoomParticipationStatus::Left,
                },
            ));
        }
    }
}
