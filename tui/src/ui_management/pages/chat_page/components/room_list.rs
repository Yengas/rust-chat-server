use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use super::super::section::usage::{HasUsageInfo, UsageInfo, UsageInfoLine};
use crate::{
    state_store::{action::Action, State},
    ui_management::pages::chat_page::section::SectionActivation,
};

use crate::ui_management::components::{Component, ComponentRender};

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
        let mut rooms = state
            .room_data_map
            .iter()
            .map(|(name, room_data)| RoomState {
                name: name.clone(),
                description: room_data.description.clone(),
                has_joined: room_data.has_joined,
                has_unread: room_data.has_unread,
            })
            .collect::<Vec<RoomState>>();

        rooms.sort_by(|room_a, room_b| room_a.name.cmp(&room_b.name));

        Self {
            rooms,
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

impl Component for RoomList {
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

    fn name(&self) -> &str {
        "Room List"
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

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
            }
            _ => (),
        }
    }
}

impl SectionActivation for RoomList {
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
}

pub struct RenderProps {
    pub border_color: Color,
    pub area: Rect,
}

impl ComponentRender<RenderProps> for RoomList {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: RenderProps) {
        let active_room = self.props.active_room.clone();
        let room_list: Vec<ListItem> = self
            .rooms()
            .iter()
            .map(|room_state| {
                let room_tag = format!(
                    "#{}{}",
                    room_state.name,
                    if room_state.has_unread { "*" } else { "" }
                );
                let content = Line::from(Span::raw(room_tag));

                let style = if self.list_state.selected().is_none()
                    && active_room.is_some()
                    && active_room.as_ref().unwrap().eq(&room_state.name)
                {
                    Style::default().add_modifier(Modifier::BOLD)
                } else if room_state.has_unread {
                    Style::default().add_modifier(Modifier::SLOW_BLINK | Modifier::ITALIC)
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style.bg(Color::Reset))
            })
            .collect();

        let room_list = List::new(room_list)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(props.border_color))
                    .title("Rooms"),
            )
            .highlight_style(
                Style::default()
                    // yellow that would work for both dark / light modes
                    .bg(Color::Rgb(255, 223, 102))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">");

        let mut app_room_list_state = self.list_state.clone();
        frame.render_stateful_widget(room_list, props.area, &mut app_room_list_state);
    }
}

impl HasUsageInfo for RoomList {
    fn usage_info(&self) -> UsageInfo {
        UsageInfo {
            description: Some("Select the room to talk in".into()),
            lines: vec![
                UsageInfoLine {
                    keys: vec!["Esc".into()],
                    description: "to cancel".into(),
                },
                UsageInfoLine {
                    keys: vec!["↑".into(), "↓".into()],
                    description: "to navigate".into(),
                },
                UsageInfoLine {
                    keys: vec!["Enter".into()],
                    description: "to join room".into(),
                },
            ],
        }
    }
}
