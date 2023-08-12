use std::{cell::RefCell, rc::Rc, sync::RwLock};

use async_trait::async_trait;
use comms::{
    command,
    event::{
        self, LoginSuccessfulReplyEvent, RoomParticipationBroacastEvent, UserMessageBroadcastEvent,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use tokio::net::tcp::OwnedWriteHalf;

use super::client::CommandWriter;

use super::{
    shared_state::SharedState,
    widget_handler::{WidgetHandler, WidgetKeyHandled, WidgetUsage, WidgetUsageKey},
};

impl SharedState {
    fn active_room_eq(&self, other: &str) -> bool {
        if let Some(active_room) = self.active_room.as_ref().map(|room| room.as_str()) {
            active_room.eq(other)
        } else {
            false
        }
    }
}

pub struct RoomState {
    pub name: String,
    pub description: String,
    pub joined: bool,
    pub has_unread: bool,
}

impl RoomState {
    fn new(name: String, description: String) -> RoomState {
        RoomState {
            name,
            description,
            joined: false,
            has_unread: false,
        }
    }
}

pub struct RoomList {
    /// Command Writer is used to send commands
    command_writer: Rc<RefCell<CommandWriter<OwnedWriteHalf>>>,
    /// Shared state between widgets
    shared_state: Rc<RwLock<SharedState>>,
    /// List with optional selection and current offset
    pub state: ListState,
    /// The list of rooms the user can participate in and their status
    pub rooms: Vec<RoomState>,
}

impl RoomList {
    pub(super) fn new(
        command_writer: Rc<RefCell<CommandWriter<OwnedWriteHalf>>>,
        shared_state: Rc<RwLock<SharedState>>,
    ) -> Self {
        Self {
            command_writer,
            shared_state,
            state: ListState::default(),
            rooms: Vec::new(),
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.rooms.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.rooms.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub(super) fn process_login_success(&mut self, event: &LoginSuccessfulReplyEvent) {
        self.rooms = event
            .rooms
            .clone()
            .into_iter()
            .map(|r| RoomState::new(r.name, r.description))
            .collect();
    }

    pub(super) fn process_user_message(&mut self, event: &UserMessageBroadcastEvent) {
        let is_for_active_room = {
            self.shared_state
                .read()
                .unwrap()
                .active_room_eq(&event.room)
        };

        if !is_for_active_room {
            if let Some(room) = self.get_room_mut(&event.room) {
                room.has_unread = true;
            }
        }
    }

    pub(super) fn process_room_participation(
        &mut self,
        event: &RoomParticipationBroacastEvent,
        username: &str,
    ) {
        if event.username == username {
            let room = self
                .get_room_mut(event.room.as_str())
                .expect("room not found");

            room.joined = match event.status {
                event::RoomParticipationStatus::Joined => true,
                event::RoomParticipationStatus::Left => false,
            };
        }
    }

    fn get_room_mut(&mut self, name: &str) -> Option<&mut RoomState> {
        self.rooms.iter_mut().find(|room| room.name == name)
    }

    fn get_room_idx_and_state(&self, name: &str) -> Option<(usize, &RoomState)> {
        self.rooms.iter().enumerate().find_map(|(idx, room)| {
            if room.name == name {
                Some((idx, room))
            } else {
                None
            }
        })
    }
}

#[async_trait(?Send)]
impl WidgetHandler for RoomList {
    fn activate(&mut self) {
        let idx: usize = self
            .shared_state
            .read()
            .ok()
            .and_then(|state| state.active_room.clone())
            .and_then(|room_name| {
                self.get_room_idx_and_state(room_name.as_str())
                    .map(|(idx, _)| idx)
            })
            .unwrap_or(0);

        *self.state.offset_mut() = 0;
        self.state.select(Some(idx));
    }

    fn deactivate(&mut self) {
        *self.state.offset_mut() = 0;
        self.state.select(None);
    }

    fn name(&self) -> &str {
        "Room List"
    }

    fn usage(&self) -> WidgetUsage {
        WidgetUsage {
            description: Some("Select the room to talk in".into()),
            keys: vec![
                WidgetUsageKey {
                    keys: vec!["Esc".into()],
                    description: "to cancel".into(),
                },
                WidgetUsageKey {
                    keys: vec!["↑".into(), "↓".into()],
                    description: "to navigate".into(),
                },
                WidgetUsageKey {
                    keys: vec!["Enter".into()],
                    description: "to join room".into(),
                },
            ],
        }
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled {
        match key.code {
            KeyCode::Up => {
                self.previous();
            }
            KeyCode::Down => {
                self.next();
            }
            KeyCode::Enter if self.state.selected().is_some() => {
                let selected_idx = self.state.selected().unwrap();
                let room_state = self.rooms.get_mut(selected_idx).unwrap();
                self.shared_state.write().unwrap().active_room = Some(room_state.name.clone());

                if !room_state.joined {
                    let _ = self
                        .command_writer
                        .borrow_mut()
                        .write(&command::UserCommand::JoinRoom(command::JoinRoomCommand {
                            room: room_state.name.clone(),
                        }))
                        .await;
                }

                room_state.has_unread = false;

                return WidgetKeyHandled::LoseFocus;
            }
            _ => (),
        }

        WidgetKeyHandled::Ok
    }
}
