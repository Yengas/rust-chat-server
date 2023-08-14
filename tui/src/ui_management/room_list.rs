use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, State};

use super::framework::widget_handler::{
    WidgetHandler, WidgetKeyHandled, WidgetUsage, WidgetUsageKey,
};

pub struct RoomState {
    pub name: String,
    pub description: String,
    pub has_joined: bool,
    pub has_unread: bool,
}

struct Props {
    /// List of rooms and current state of those rooms
    rooms: Vec<RoomState>,
    /// Current active room
    active_room: Option<String>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Self {
            rooms: state
                .room_data_map
                .iter()
                .map(|(name, room_data)| RoomState {
                    name: name.clone(),
                    description: room_data.description.clone(),
                    has_joined: room_data.has_joined,
                    // TODO: fix has unread
                    has_unread: false,
                })
                .collect(),
            active_room: state.active_room.clone(),
        }
    }
}

pub struct RoomList {
    /// Sending actions to the state store
    action_tx: UnboundedSender<Action>,
    /// State Mapped RoomList Props
    props: Props,
    // Internal Component State
    /// List with optional selection and current offset
    pub list_state: ListState,
}

impl RoomList {
    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.props.rooms.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.props.rooms.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i));
    }

    pub(super) fn rooms(&self) -> &Vec<RoomState> {
        &self.props.rooms
    }

    fn get_room_idx(&self, name: &str) -> Option<usize> {
        self.props
            .rooms
            .iter()
            .enumerate()
            .find_map(|(idx, room_state)| {
                if room_state.name == name {
                    Some(idx)
                } else {
                    None
                }
            })
    }
}

impl WidgetHandler for RoomList {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self {
        Self {
            action_tx,
            props: Props::from(state),
            //
            list_state: ListState::default(),
        }
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        Self {
            props: Props::from(state),
            ..self
        }
    }

    fn activate(&mut self) {
        let idx: usize = self
            .props
            .active_room
            .as_ref()
            .and_then(|room_name| self.get_room_idx(room_name.as_str()))
            .unwrap_or(0);

        *self.list_state.offset_mut() = 0;
        self.list_state.select(Some(idx));
    }

    fn deactivate(&mut self) {
        *self.list_state.offset_mut() = 0;
        self.list_state.select(None);
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

    fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled {
        match key.code {
            KeyCode::Up => {
                self.previous();
            }
            KeyCode::Down => {
                self.next();
            }
            KeyCode::Enter if self.list_state.selected().is_some() => {
                let selected_idx = self.list_state.selected().unwrap();

                let rooms = self.rooms();
                let room_state = rooms.get(selected_idx).unwrap();

                // TODO: handle the error scenario somehow
                let _ = self.action_tx.send(Action::SelectRoom {
                    room: room_state.name.clone(),
                });

                return WidgetKeyHandled::LoseFocus;
            }
            _ => (),
        }

        WidgetKeyHandled::Ok
    }
}
