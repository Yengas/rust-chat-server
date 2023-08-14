use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, RoomData, State};

use super::{
    framework::widget_handler::{WidgetHandler, WidgetKeyHandled, WidgetUsage, WidgetUsageKey},
    message_input_box::MessageInputBox,
    room_list::RoomList,
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

struct Props {
    /// The logged in user
    username: String,
    /// The currently active room
    active_room: Option<String>,
    /// The timer for the chat page
    timer: usize,
    /// The room data map
    room_data_map: HashMap<String, RoomData>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Props {
            username: state.username.clone(),
            active_room: state.active_room.clone(),
            timer: state.timer,
            room_data_map: state.room_data_map.clone(),
        }
    }
}

const DEFAULT_HOVERED_SECTION: Section = Section::MessageInput;

/// ChatPage handles the UI and the state of the chat page
pub struct ChatPage {
    /// Action sender
    pub action_tx: UnboundedSender<Action>,
    /// State Mapped ChatPage Props
    props: Props,
    // Internal State
    /// Currently active section, handling input
    pub active_section: Option<Section>,
    /// Section that is currently hovered
    pub last_hovered_section: Section,
    // Child Components
    /// The room list widget that handles the listing of the rooms
    pub room_list: RoomList,
    /// The input box widget that handles the message input
    pub input_box: MessageInputBox,
}

impl ChatPage {
    pub(super) fn username(&self) -> &String {
        &self.props.username
    }

    pub(super) fn active_room(&self) -> &Option<String> {
        &self.props.active_room
    }

    pub(super) fn timer(&self) -> usize {
        self.props.timer
    }

    pub(super) fn get_room_data(&self, name: &str) -> Option<&RoomData> {
        self.props.room_data_map.get(name)
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
}

impl WidgetHandler for ChatPage {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self
    where
        Self: Sized,
    {
        ChatPage {
            action_tx: action_tx.clone(),
            // set the props
            props: Props::from(state),
            // internal component state
            active_section: Option::None,
            last_hovered_section: DEFAULT_HOVERED_SECTION,
            // child components
            room_list: RoomList::new(state, action_tx.clone()),
            input_box: MessageInputBox::new(state, action_tx),
        }
        .move_with_state(state)
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        ChatPage {
            props: Props::from(state),
            // propogate the update to the child components
            room_list: self.room_list.move_with_state(state),
            input_box: self.input_box.move_with_state(state),
            ..self
        }
    }

    fn name(&self) -> &str {
        "Chat Page"
    }

    fn activate(&mut self) {}
    fn deactivate(&mut self) {}

    fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled {
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
                    let _ = self.action_tx.send(Action::Exit);
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = self.action_tx.send(Action::Exit);
                }
                _ => {}
            },
            Some(section) if key.code == KeyCode::Esc => {
                self.get_handler_for_section_mut(&section).deactivate();
                self.active_section = None;
            }
            Some(section) => {
                let handler = self.get_handler_for_section_mut(&section);

                if let WidgetKeyHandled::LoseFocus = handler.handle_key_event(key) {
                    handler.deactivate();

                    self.active_section = None;
                }
            }
        }

        WidgetKeyHandled::Ok
    }

    fn usage(&self) -> WidgetUsage {
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
