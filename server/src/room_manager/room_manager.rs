use std::{collections::HashMap, sync::Arc};

use comms::event::Event;
use tokio::sync::{broadcast, Mutex};

use super::room::{ChatRoom, ChatRoomMetadata, SessionAndUserId, UserSessionHandle};

pub type RoomJoinResult = (broadcast::Receiver<Event>, UserSessionHandle, Vec<String>);

#[derive(Debug, Clone)]
pub struct RoomManager {
    chat_rooms: HashMap<String, Arc<Mutex<ChatRoom>>>,
    chat_room_metadatas: Vec<ChatRoomMetadata>,
}

impl RoomManager {
    pub(super) fn new(chat_rooms: Vec<(ChatRoomMetadata, Arc<Mutex<ChatRoom>>)>) -> RoomManager {
        let chat_room_metadatas = chat_rooms
            .iter()
            .map(|(metadata, _)| metadata.clone())
            .collect();

        RoomManager {
            chat_room_metadatas,
            chat_rooms: chat_rooms
                .into_iter()
                .map(|(metadata, chat_room)| (metadata.name.clone(), chat_room))
                .collect(),
        }
    }

    pub fn chat_room_metadatas(&self) -> &Vec<ChatRoomMetadata> {
        &self.chat_room_metadatas
    }

    /// Joins to a room given a user session
    pub async fn join_room(
        &self,
        room_name: &str,
        session_and_user_id: &SessionAndUserId,
    ) -> anyhow::Result<RoomJoinResult> {
        let room = self
            .chat_rooms
            .get(room_name)
            .ok_or_else(|| anyhow::anyhow!("room '{}' not found", room_name))?;

        let mut room = room.lock().await;
        let (broadcast_rx, user_session_handle) = room.join(session_and_user_id);

        Ok((
            broadcast_rx,
            user_session_handle,
            room.get_unique_user_ids().clone(),
        ))
    }

    pub async fn drop_user_session_handle(&self, handle: UserSessionHandle) -> anyhow::Result<()> {
        let room = self
            .chat_rooms
            .get(handle.room())
            .ok_or_else(|| anyhow::anyhow!("room '{}' not found", handle.room()))?;

        let mut room = room.lock().await;

        room.leave(handle);

        Ok(())
    }
}
