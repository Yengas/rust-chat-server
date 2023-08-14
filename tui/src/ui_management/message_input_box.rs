use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, State};

use super::framework::widget_handler::{
    WidgetHandler, WidgetKeyHandled, WidgetUsage, WidgetUsageKey,
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

impl WidgetHandler for MessageInputBox {
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

    fn usage(&self) -> WidgetUsage {
        if self.props.active_room.is_none() {
            WidgetUsage {
                description: Some("You can not send a message until you enter a room.".into()),
                keys: vec![WidgetUsageKey {
                    keys: vec!["Esc".into()],
                    description: "to cancel".into(),
                }],
            }
        } else {
            WidgetUsage {
                description: Some("Type your message to send a message to the active room".into()),
                keys: vec![
                    WidgetUsageKey {
                        keys: vec!["Esc".into()],
                        description: "to cancel".into(),
                    },
                    WidgetUsageKey {
                        keys: vec!["Enter".into()],
                        description: "to send your message".into(),
                    },
                ],
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled {
        if key.kind != KeyEventKind::Press {
            return WidgetKeyHandled::Ok;
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

        WidgetKeyHandled::Ok
    }
}
