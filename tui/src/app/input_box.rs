use std::{cell::RefCell, rc::Rc, sync::RwLock};

use async_trait::async_trait;
use comms::command;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::net::tcp::OwnedWriteHalf;

use super::client::CommandWriter;

use super::{
    shared_state::SharedState,
    widget_handler::{WidgetHandler, WidgetKeyHandled, WidgetUsage, WidgetUsageKey},
};

pub struct InputBox {
    command_writer: Rc<RefCell<CommandWriter<OwnedWriteHalf>>>,
    /// Shared state between widgets
    shared_state: Rc<RwLock<SharedState>>,
    /// Current value of the input box
    pub text: String,
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
}

impl InputBox {
    pub(super) fn new(
        command_writer: Rc<RefCell<CommandWriter<OwnedWriteHalf>>>,
        shared_state: Rc<RwLock<SharedState>>,
    ) -> Self {
        Self {
            command_writer,
            shared_state,
            text: String::new(),
            cursor_position: 0,
        }
    }

    async fn submit_message(&mut self, room: String) {
        // TODO: handle the promise
        let _ = {
            self.command_writer
                .borrow_mut()
                .write(&command::UserCommand::SendMessage(
                    command::SendMessageCommand {
                        room,
                        content: self.text.clone(),
                    },
                ))
        }
        .await;

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

#[async_trait(?Send)]
impl WidgetHandler for InputBox {
    fn activate(&mut self) {}

    fn deactivate(&mut self) {
        self.cursor_position = 0;
        self.text.clear();
    }

    fn name(&self) -> &str {
        "Message Input"
    }

    fn usage(&self) -> WidgetUsage {
        if self.shared_state.read().unwrap().active_room.is_none() {
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

    async fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled {
        if key.kind != KeyEventKind::Press {
            return WidgetKeyHandled::Ok;
        }
        let active_room = self.shared_state.read().unwrap().active_room.clone();
        if let Some(active_room) = active_room {
            match key.code {
                KeyCode::Enter => {
                    self.submit_message(active_room).await;
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
