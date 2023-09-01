use std::sync::Arc;

use tokio::sync::Mutex;

use self::room::{ChatRoom, ChatRoomMetadata};
pub use self::room::{SessionAndUserId, UserSessionHandle};

pub use self::room_manager::RoomManager;

mod room;
#[allow(clippy::module_inception)]
mod room_manager;

#[derive(Debug)]
pub struct RoomManagerBuilder {
    chat_rooms: Vec<(ChatRoomMetadata, Arc<Mutex<room::ChatRoom>>)>,
}

impl RoomManagerBuilder {
    pub fn new() -> Self {
        RoomManagerBuilder {
            chat_rooms: Vec::new(),
        }
    }

    /// Add a room to the room manager
    /// Will panic if a room with the same name already exists
    pub fn create_room(mut self, name: &str, description: &str) -> Self {
        let metadata = ChatRoomMetadata::new(name, description);
        let chat_room = Arc::new(Mutex::new(ChatRoom::new(metadata.clone())));

        if self
            .chat_rooms
            .iter()
            .any(|(metadata, _)| metadata.name.eq(name))
        {
            panic!("room with the same name already exists");
        }

        self.chat_rooms.push((metadata, chat_room));

        self
    }

    pub fn build(self) -> RoomManager {
        RoomManager::new(self.chat_rooms)
    }
}
