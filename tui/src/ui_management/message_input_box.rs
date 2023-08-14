use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    state_store::{action::Action, State},
    ui_management::framework::usage::{UsageInfo, UsageInfoLine},
};

use super::framework::{
    component::{Component, ComponentKeyHandled, ComponentRender},
    usage::HasUsageInfo,
};

struct Props {
    /// Active room that the user is chatting in
    active_room: Option<String>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Self {
            active_room: state.active_room.clone(),
        }
    }
}

pub struct MessageInputBox {
    action_tx: UnboundedSender<Action>,
    /// State Mapped MessageInputBox Props
    props: Props,
    // Internal State for the Component
    /// Current value of the input box
    pub text: String,
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
}

impl MessageInputBox {
    fn submit_message(&mut self) {
        // TODO: handle the error scenario
        let _ = self.action_tx.send(Action::SendMessage {
            content: self.text.clone(),
        });

        self.text.clear();
        self.reset_cursor();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.text.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.text.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.text.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.text.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
}

impl Component for MessageInputBox {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self {
        Self {
            action_tx,
            props: Props::from(state),
            //
            text: String::new(),
            cursor_position: 0,
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

    fn activate(&mut self) {}

    fn deactivate(&mut self) {
        self.cursor_position = 0;
        self.text.clear();
    }

    fn name(&self) -> &str {
        "Message Input"
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> ComponentKeyHandled {
        if key.kind != KeyEventKind::Press {
            return ComponentKeyHandled::Ok;
        }

        if self.props.active_room.is_some() {
            match key.code {
                KeyCode::Enter => {
                    self.submit_message();
                }
                KeyCode::Char(to_insert) => {
                    self.enter_char(to_insert);
                }
                KeyCode::Backspace => {
                    self.delete_char();
                }
                KeyCode::Left => {
                    self.move_cursor_left();
                }
                KeyCode::Right => {
                    self.move_cursor_right();
                }
                _ => {}
            }
        }

        ComponentKeyHandled::Ok
    }
}

pub struct RenderProps {
    pub area: Rect,
    pub border_color: Color,
    pub show_cursor: bool,
}

impl ComponentRender<RenderProps> for MessageInputBox {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: RenderProps) {
        let input = Paragraph::new(self.text.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .fg(props.border_color)
                    .title("Input"),
            );
        frame.render_widget(input, props.area);

        // Cursor is hidden by default, so we need to make it visible if the input box is selected
        if props.show_cursor {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            frame.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                props.area.x + self.cursor_position as u16 + 1,
                // Move one line down, from the border to the input line
                props.area.y + 1,
            )
        }
    }
}

impl HasUsageInfo for MessageInputBox {
    fn usage_info(&self) -> UsageInfo {
        if self.props.active_room.is_none() {
            UsageInfo {
                description: Some("You can not send a message until you enter a room.".into()),
                lines: vec![UsageInfoLine {
                    keys: vec!["Esc".into()],
                    description: "to cancel".into(),
                }],
            }
        } else {
            UsageInfo {
                description: Some("Type your message to send a message to the active room".into()),
                lines: vec![
                    UsageInfoLine {
                        keys: vec!["Esc".into()],
                        description: "to cancel".into(),
                    },
                    UsageInfoLine {
                        keys: vec!["Enter".into()],
                        description: "to send your message".into(),
                    },
                ],
            }
        }
    }
}
