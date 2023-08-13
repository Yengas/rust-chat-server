use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::{
    app::{action::Action, RoomData, State},
    Interrupted, Terminator,
};

use super::{
    message_input_box::MessageInputBox,
    room_list::RoomList,
    widget_handler::{WidgetHandler, WidgetKeyHandled, WidgetUsage, WidgetUsageKey},
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

const DEFAULT_HOVERED_SECTION: Section = Section::MessageInput;

/// ChatPage handles the UI and the state of the chat page
pub struct ChatPage {
    /// Terminator is used to send the kill signal to the application
    terminator: Terminator,
    /// Shared state between widgets
    pub state: Rc<RefCell<State>>,
    /// Currently active section, handling input
    pub active_section: Option<Section>,
    /// Section that is currently hovered
    pub last_hovered_section: Section,
    // The room list widget that handles the listing of the rooms
    pub room_list: RoomList,
    // The input box widget that handles the message input
    pub input_box: MessageInputBox,
}

impl ChatPage {
    pub fn new(
        terminator: Terminator,
        action_tx: mpsc::UnboundedSender<Action>,
        state: Rc<RefCell<State>>,
    ) -> Self {
        ChatPage {
            terminator,
            state: Rc::clone(&state),
            //
            active_section: Option::None,
            last_hovered_section: DEFAULT_HOVERED_SECTION,
            //
            room_list: RoomList::new(action_tx.clone(), Rc::clone(&state)),
            input_box: MessageInputBox::new(action_tx, Rc::clone(&state)),
        }
    }

    pub(super) fn username(&self) -> String {
        self.state.borrow().username.clone()
    }

    pub(super) fn active_room(&self) -> Option<String> {
        self.state.borrow().active_room.clone()
    }

    pub(super) fn timer(&self) -> usize {
        self.state.borrow().timer
    }

    pub(super) fn get_room_data(&self, name: &str) -> Option<RoomData> {
        self.state.borrow().room_data_map.get(name).cloned()
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
