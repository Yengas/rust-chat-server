use std::collections::{HashMap, HashSet};

use circular_queue::CircularQueue;
use comms::event;

#[derive(Debug, Clone)]
pub enum MessageBoxItem {
    Message { user_id: String, content: String },
    Notification(String),
}

const MAX_MESSAGES_TO_STORE_PER_ROOM: usize = 100;

/// RoomData holds the data for a room
#[derive(Debug, Clone)]
pub struct RoomData {
    /// The name of the room
    pub name: String,
    /// The description of the Room
    pub description: String,
    /// List of users in the room
    pub users: HashSet<String>,
    /// History of recorded messages
    pub messages: CircularQueue<MessageBoxItem>,
    /// Has joined the room
    pub has_joined: bool,
    /// Has unread messages
    pub has_unread: bool,
}

impl Default for RoomData {
    fn default() -> Self {
        RoomData {
            name: String::new(),
            description: String::new(),
            users: HashSet::new(),
            messages: CircularQueue::with_capacity(MAX_MESSAGES_TO_STORE_PER_ROOM),
            has_joined: false,
            has_unread: false,
        }
    }
}

impl RoomData {
    pub fn new(name: String, description: String) -> Self {
        RoomData {
            name,
            description,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum ServerConnectionStatus {
    Uninitalized,
    Connecting,
    Connected { addr: String },
    Errored { err: String },
}

/// State holds the state of the application
#[derive(Debug, Clone)]
pub struct State {
    pub server_connection_status: ServerConnectionStatus,
    /// Currently active room
    pub active_room: Option<String>,
    /// The id of the user
    pub user_id: String,
    /// Storage of room data
    pub room_data_map: HashMap<String, RoomData>,
    /// Timer since app was opened
    pub timer: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            server_connection_status: ServerConnectionStatus::Uninitalized,
            active_room: None,
            user_id: String::new(),
            room_data_map: HashMap::new(),
            timer: 0,
        }
    }
}

impl State {
    pub fn handle_server_event(&mut self, event: &event::Event) {
        match event {
            event::Event::LoginSuccessful(event) => {
                self.user_id = event.user_id.clone();
                self.room_data_map = event
                    .rooms
                    .clone()
                    .into_iter()
                    .map(|r| (r.name.clone(), RoomData::new(r.name, r.description)))
                    .collect();
            }
            event::Event::RoomParticipation(event) => {
                if let Some(room_data) = self.room_data_map.get_mut(&event.room) {
                    match event.status {
                        event::RoomParticipationStatus::Joined => {
                            room_data.users.insert(event.user_id.clone());
                            if event.user_id == self.user_id {
                                room_data.has_joined = true;
                            }
                        }
                        event::RoomParticipationStatus::Left => {
                            room_data.users.remove(&event.user_id);
                            if event.user_id == self.user_id {
                                room_data.has_joined = false;
                            }
                        }
                    }

                    room_data
                        .messages
                        .push(MessageBoxItem::Notification(format!(
                            "{} has {} the room",
                            event.user_id,
                            match event.status {
                                event::RoomParticipationStatus::Joined => "joined",
                                event::RoomParticipationStatus::Left => "left",
                            }
                        )));
                }
            }
            event::Event::UserJoinedRoom(event) => {
                self.room_data_map.get_mut(&event.room).unwrap().users =
                    event.users.clone().into_iter().collect();
            }
            event::Event::UserMessage(event) => {
                let room_data = self.room_data_map.get_mut(&event.room).unwrap();

                room_data.messages.push(MessageBoxItem::Message {
                    user_id: event.user_id.clone(),
                    content: event.content.clone(),
                });

                if let Some(active_room) = self.active_room.as_ref() {
                    if !active_room.eq(&event.room) {
                        room_data.has_unread = true;
                    }
                }
            }
        }
    }

    pub fn mark_connection_request_start(&mut self) {
        self.server_connection_status = ServerConnectionStatus::Connecting;
    }

    /// Processes the result of a connection request to change the state of the application
    pub fn process_connection_request_result(&mut self, result: anyhow::Result<String>) {
        self.server_connection_status = match result {
            Ok(addr) => ServerConnectionStatus::Connected { addr: addr.clone() },
            Err(err) => ServerConnectionStatus::Errored {
                err: err.to_string(),
            },
        }
    }

    /// Tries to set the active room as the given room. Returns the [RoomData] associated to the room.
    pub fn try_set_active_room(&mut self, room: &str) -> Option<&RoomData> {
        let room_data = self.room_data_map.get_mut(room)?;
        room_data.has_unread = false;

        self.active_room = Some(String::from(room));

        Some(room_data)
    }

    pub fn tick_timer(&mut self) {
        self.timer += 1;
    }
}
