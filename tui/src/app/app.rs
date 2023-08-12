use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::RwLock,
};

use comms::event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::net::tcp::OwnedWriteHalf;

use crate::{Interrupted, Terminator};

use super::{
    client::CommandWriter,
    input_box::InputBox,
    room_list::RoomList,
    shared_state::SharedState,
    widget_handler::{WidgetHandler, WidgetKeyHandled, WidgetUsageKey},
    WidgetUsage,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    MessageInput,
    RoomList,
}

impl Section {
    pub const COUNT: usize = 2;

    fn to_usize(&self) -> usize {
        match self {
            Section::MessageInput => 0,
            Section::RoomList => 1,
        }
    }
}

impl TryFrom<usize> for Section {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Section::MessageInput),
            1 => Ok(Section::RoomList),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MessageBoxItem {
    Message { username: String, content: String },
    Notification(String),
}

/// RoomData holds the data for a room
#[derive(Debug, Default, Clone)]
pub struct RoomData {
    /// List of users in the room
    pub users: HashSet<String>,
    /// History of recorded messages
    pub messages: Vec<MessageBoxItem>,
}

const DEFAULT_HOVERED_SECTION: Section = Section::MessageInput;

/// App holds the state of the application
pub struct App {
    /// Terminator is used to send the kill signal to the application
    terminator: Terminator,
    /// Shared state between widgets
    pub shared_state: Rc<RwLock<SharedState>>,
    /// Currently active section, handling input
    pub active_section: Option<Section>,
    /// Section that is currently hovered
    pub last_hovered_section: Section,
    /// The name of the user
    pub username: String,
    // The room list widget that handles the listing of the rooms
    pub room_list: RoomList,
    // The input box widget that handles the message input
    pub input_box: InputBox,
    /// Storage of room data
    pub room_data_map: HashMap<String, RoomData>,
    /// Timer since app was open
    pub timer: usize,
}

impl App {
    pub fn new(command_writer: CommandWriter<OwnedWriteHalf>, terminator: Terminator) -> Self {
        let shared_state = Rc::new(RwLock::new(SharedState::new()));
        let command_writer_1 = Rc::new(RefCell::new(command_writer));
        let command_writer_2 = Rc::clone(&command_writer_1);

        App {
            terminator,
            shared_state: Rc::clone(&shared_state),
            active_section: Option::None,
            last_hovered_section: DEFAULT_HOVERED_SECTION,
            //
            username: String::new(),
            //
            room_list: RoomList::new(command_writer_1, Rc::clone(&shared_state)),
            //
            input_box: InputBox::new(command_writer_2, Rc::clone(&shared_state)),
            //
            room_data_map: HashMap::new(),
            timer: 0,
        }
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        let active_section = self.active_section.clone();
        match active_section {
            None => match key.code {
                KeyCode::Char('e') => {
                    let last_hovered_section = self.last_hovered_section.clone();

                    self.active_section = Some(last_hovered_section.clone());
                    self.get_handler_for_section_mut(&last_hovered_section)
                        .activate();
                }
                KeyCode::Left => self.hover_previous(),
                KeyCode::Right => self.hover_next(),
                KeyCode::Char('q') => {
                    let _ = self.terminator.terminate(Interrupted::UserInt);
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = self.terminator.terminate(Interrupted::UserInt);
                }
                _ => {}
            },
            Some(section) if key.code == KeyCode::Esc => {
                self.get_handler_for_section_mut(&section).deactivate();
                self.active_section = None;
            }
            Some(section) => {
                let handler = self.get_handler_for_section_mut(&section);

                if let WidgetKeyHandled::LoseFocus = handler.handle_key_event(key).await {
                    handler.deactivate();

                    self.active_section = None;
                }
            }
        }
    }

    pub(super) fn handle_server_disconnect(&mut self) -> anyhow::Result<Interrupted> {
        self.terminator.terminate(Interrupted::ServerDisconnected)?;
        Ok(Interrupted::ServerDisconnected)
    }

    fn get_handler_for_section<'a>(&'a self, section: &Section) -> &'a dyn WidgetHandler {
        match section {
            Section::MessageInput => &self.input_box,
            Section::RoomList => &self.room_list,
        }
    }

    fn get_handler_for_section_mut<'a>(
        &'a mut self,
        section: &Section,
    ) -> &'a mut dyn WidgetHandler {
        match section {
            Section::MessageInput => &mut self.input_box,
            Section::RoomList => &mut self.room_list,
        }
    }

    pub(super) fn handle_server_event(&mut self, event: &event::Event) {
        match event {
            event::Event::LoginSuccessful(event) => {
                self.username = event.username.clone();
                self.room_list.process_login_success(event);
                self.room_data_map = event
                    .rooms
                    .clone()
                    .into_iter()
                    .map(|r| (r.name, RoomData::default()))
                    .collect();
            }
            event::Event::RoomParticipation(event) => {
                self.room_list
                    .process_room_participation(event, self.username.as_str());
                let room_data = self.room_data_map.get_mut(&event.room).unwrap();

                match event.status {
                    event::RoomParticipationStatus::Joined => {
                        room_data.users.insert(event.username.clone());
                    }
                    event::RoomParticipationStatus::Left => {
                        room_data.users.remove(&event.username);
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

                self.room_list.process_user_message(event);
            }
        }
    }

    fn hover_next(&mut self) {
        let idx: usize = self.last_hovered_section.to_usize();
        let next_idx = (idx + 1) % Section::COUNT;
        self.last_hovered_section = Section::try_from(next_idx).unwrap();
    }

    fn hover_previous(&mut self) {
        let idx: usize = self.last_hovered_section.to_usize();
        let previous_idx = if idx == 0 {
            Section::COUNT - 1
        } else {
            idx - 1
        };
        self.last_hovered_section = Section::try_from(previous_idx).unwrap();
    }

    pub(super) fn increment_timer(&mut self) {
        self.timer += 1;
    }

    pub fn usage(&self) -> WidgetUsage {
        if let Some(section) = self.active_section.as_ref() {
            let handler: &dyn WidgetHandler = match section {
                Section::RoomList => &self.room_list,
                Section::MessageInput => &self.input_box,
            };

            handler.usage()
        } else {
            WidgetUsage {
                description: Some("Select a widget".into()),
                keys: vec![
                    WidgetUsageKey {
                        keys: vec!["q".into()],
                        description: "to exit".into(),
                    },
                    WidgetUsageKey {
                        keys: vec!["←".into(), "→".into()],
                        description: "to hover widgets".into(),
                    },
                    WidgetUsageKey {
                        keys: vec!["e".into()],
                        description: format!(
                            "to activate {}",
                            self.get_handler_for_section(&self.last_hovered_section)
                                .name()
                        ),
                    },
                ],
            }
        }
    }
}
