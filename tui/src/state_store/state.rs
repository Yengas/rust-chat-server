use std::collections::{HashMap, HashSet};

use comms::event;

#[derive(Debug, Clone)]
pub enum MessageBoxItem {
    Message { username: String, content: String },
    Notification(String),
}

/// RoomData holds the data for a room
#[derive(Debug, Default, Clone)]
pub struct RoomData {
    /// The name of the room
    pub name: String,
    /// The description of the Room
    pub description: String,
    /// List of users in the room
    pub users: HashSet<String>,
    /// History of recorded messages
    pub messages: Vec<MessageBoxItem>,
    /// Has joined the room
    pub has_joined: bool,
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
    /// The name of the user
    pub username: String,
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
            username: String::new(),
            room_data_map: HashMap::new(),
            timer: 0,
        }
    }
}

impl State {
    pub(super) fn handle_server_event(&mut self, event: &event::Event) {
        match event {
            event::Event::LoginSuccessful(event) => {
                self.username = event.username.clone();
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
                            room_data.users.insert(event.username.clone());
                            if event.username == self.username {
                                room_data.has_joined = true;
                            }
                        }
                        event::RoomParticipationStatus::Left => {
                            room_data.users.remove(&event.username);
                            if event.username == self.username {
                                room_data.has_joined = false;
                            }
                        }
                    }

                    room_data
                        .messages
                        .push(MessageBoxItem::Notification(format!(
                            "{} has {} the room",
                            event.username,
                            match event.status {
                                event::RoomParticipationStatus::Joined => "joined",
                                event::RoomParticipationStatus::Left => "left",
                            }
                        )));
                }
            }
            event::Event::UserJoinedRoom(event) => {
                self.room_data_map.get_mut(&event.room).unwrap().users = event.users.clone();
            }
            event::Event::UserMessage(event) => {
                self.room_data_map
                    .get_mut(&event.room)
                    .unwrap()
                    .messages
                    .push(MessageBoxItem::Message {
                        username: event.username.clone(),
                        content: event.content.clone(),
                    });
            }
        }
    }
}
