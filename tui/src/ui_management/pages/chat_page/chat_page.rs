use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, MessageBoxItem, RoomData, State};

use super::{
    components::{
        message_input_box::{self, MessageInputBox},
        room_list::{self, RoomList},
    },
    section::{
        usage::{widget_usage_to_text, HasUsageInfo, UsageInfo, UsageInfoLine},
        SectionActivation,
    },
};
use crate::ui_management::components::{Component, ComponentRender};

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
    fn get_room_data(&self, name: &str) -> Option<&RoomData> {
        self.props.room_data_map.get(name)
    }

    fn get_component_for_section<'a>(&'a self, section: &Section) -> &'a dyn Component {
        match section {
            Section::MessageInput => &self.input_box,
            Section::RoomList => &self.room_list,
        }
    }

    fn get_component_for_section_mut<'a>(&'a mut self, section: &Section) -> &'a mut dyn Component {
        match section {
            Section::MessageInput => &mut self.input_box,
            Section::RoomList => &mut self.room_list,
        }
    }

    fn get_section_activation_for_section<'a>(
        &'a mut self,
        section: &Section,
    ) -> &'a mut dyn SectionActivation {
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

    fn calculate_border_color(&self, section: Section) -> Color {
        match (self.active_section.as_ref(), &self.last_hovered_section) {
            (Some(active_section), _) if active_section.eq(&section) => Color::Yellow,
            (_, last_hovered_section) if last_hovered_section.eq(&section) => Color::Blue,
            _ => Color::Reset,
        }
    }
}

impl Component for ChatPage {
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

    fn handle_key_event(&mut self, key: KeyEvent) {
        let active_section = self.active_section.clone();

        match active_section {
            None => match key.code {
                KeyCode::Char('e') => {
                    let last_hovered_section = self.last_hovered_section.clone();

                    self.active_section = Some(last_hovered_section.clone());
                    self.get_section_activation_for_section(&last_hovered_section)
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
            Some(section) => {
                self.get_component_for_section_mut(&section)
                    .handle_key_event(key);

                if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                    self.get_section_activation_for_section(&section)
                        .deactivate();

                    self.active_section = None;
                }
            }
        }
    }
}

const NO_ROOM_SELECTED_MESSAGE: &str = "Join at least one room to start chatting!";

// TODO: move the message list to listview and make it scrollable
fn calculate_message_list_offset(height: u16, messages_len: usize) -> usize {
    // minus 2 for borders
    messages_len.saturating_sub(height as usize - 2)
}

impl ComponentRender<()> for ChatPage {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, _props: ()) {
        let [left, middle, right] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(frame.size())
        else {
            panic!("The main layout should have 3 chunks")
        };

        let [container_room_list, container_user_info] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(4)].as_ref())
            .split(left)
        else {
            panic!("The left layout should have 2 chunks")
        };

        self.room_list.render(
            frame,
            room_list::RenderProps {
                border_color: self.calculate_border_color(Section::RoomList),
                area: container_room_list,
            },
        );

        let user_info = Paragraph::new(Text::from(vec![
            Line::from(format!("User: @{}", self.props.username)),
            Line::from(format!("Chatting for: {} secs", self.props.timer)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("User Information"),
        );
        frame.render_widget(user_info, container_user_info);

        let [container_highlight, container_messages, container_input] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(middle)
        else {
            panic!("The middle layout should have 3 chunks")
        };

        let top_line = if let Some(room_data) = self
            .props
            .active_room
            .as_ref()
            .and_then(|active_room| self.get_room_data(active_room))
        {
            Line::from(vec![
                "on ".into(),
                Span::from(format!("#{}", room_data.name)).bold(),
                " for ".into(),
                Span::from(format!(r#""{}""#, room_data.description)).italic(),
            ])
        } else {
            Line::from(NO_ROOM_SELECTED_MESSAGE)
        };
        let text = Text::from(top_line);

        let help_message = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Active Room Information"),
        );
        frame.render_widget(help_message, container_highlight);

        let messages = if let Some(active_room) = self.props.active_room.as_ref() {
            self.get_room_data(active_room)
                .map(|room_data| {
                    let message_offset = calculate_message_list_offset(
                        container_messages.height,
                        room_data.messages.len(),
                    );

                    room_data.messages[message_offset..]
                        .iter()
                        .map(|mbi| {
                            let line = match mbi {
                                MessageBoxItem::Message { username, content } => {
                                    Line::from(Span::raw(format!("@{}: {}", username, content)))
                                }
                                MessageBoxItem::Notification(content) => {
                                    Line::from(Span::raw(content.clone()).italic())
                                }
                            };

                            ListItem::new(line)
                        })
                        .collect::<Vec<ListItem>>()
                })
                .unwrap_or_default()
        } else {
            vec![ListItem::new(Line::from(NO_ROOM_SELECTED_MESSAGE))]
        };
        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
        frame.render_widget(messages, container_messages);

        self.input_box.render(
            frame,
            message_input_box::RenderProps {
                border_color: self.calculate_border_color(Section::MessageInput),
                area: container_input,
                show_cursor: self
                    .active_section
                    .as_ref()
                    .map(|active_section| active_section.eq(&Section::MessageInput))
                    .unwrap_or(false),
            },
        );

        let [container_room_users, container_usage] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(10)].as_ref())
            .split(right)
        else {
            panic!("The left layout should have 2 chunks")
        };

        let room_users_list_items: Vec<ListItem> =
            if let Some(active_room) = self.props.active_room.as_ref() {
                self.get_room_data(active_room)
                    .map(|room_data| {
                        room_data
                            .users
                            .iter()
                            .map(|user_name| {
                                ListItem::new(Line::from(Span::raw(format!("@{user_name}"))))
                            })
                            .collect::<Vec<ListItem<'_>>>()
                    })
                    .unwrap_or_default()
            } else {
                vec![]
            };

        let room_users_list = List::new(room_users_list_items)
            .block(Block::default().borders(Borders::ALL).title("Room Users"));

        frame.render_widget(room_users_list, container_room_users);

        let mut usage_text: Text = widget_usage_to_text(self.usage_info());
        usage_text.patch_style(Style::default().add_modifier(Modifier::RAPID_BLINK));
        let usage = Paragraph::new(usage_text)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Usage"));
        frame.render_widget(usage, container_usage);
    }
}

impl HasUsageInfo for ChatPage {
    fn usage_info(&self) -> UsageInfo {
        if let Some(section) = self.active_section.as_ref() {
            let handler: &dyn HasUsageInfo = match section {
                Section::RoomList => &self.room_list,
                Section::MessageInput => &self.input_box,
            };

            handler.usage_info()
        } else {
            UsageInfo {
                description: Some("Select a widget".into()),
                lines: vec![
                    UsageInfoLine {
                        keys: vec!["q".into()],
                        description: "to exit".into(),
                    },
                    UsageInfoLine {
                        keys: vec!["←".into(), "→".into()],
                        description: "to hover widgets".into(),
                    },
                    UsageInfoLine {
                        keys: vec!["e".into()],
                        description: format!(
                            "to activate {}",
                            self.get_component_for_section(&self.last_hovered_section)
                                .name()
                        ),
                    },
                ],
            }
        }
    }
}
