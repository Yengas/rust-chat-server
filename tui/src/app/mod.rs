use std::{sync::Arc, time::Duration};

use comms::{command, event::UserMessageEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tokio::sync::{broadcast, RwLock};

use crate::client::Client;

use self::termination::{Interrupted, Terminator};

pub(crate) mod termination;

pub(crate) enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
pub(crate) struct App {
    /// Client is used to send commands
    client: Client,
    /// Terminator is used to send the kill signal to the application
    terminator: Terminator,
    /// Current value of the input box
    pub(crate) input: String,
    /// Position of cursor in the editor area.
    pub(crate) cursor_position: usize,
    /// Current input mode
    pub(crate) input_mode: InputMode,
    /// History of recorded messages
    pub(crate) messages: Vec<UserMessageEvent>,
    /// Timer since app was open
    pub(crate) timer: usize,
}

impl App {
    pub fn new(client: Client, terminator: Terminator) -> App {
        App {
            client,
            terminator,
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            cursor_position: 0,
            timer: 0,
        }
    }

    pub(crate) async fn handle_key_event(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('e') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('q') => {
                    let _ = self.terminator.terminate(Interrupted::UserInt);
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = self.terminator.terminate(Interrupted::UserInt);
                }
                _ => {}
            },
            InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => self.submit_message().await,
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
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_server_event(&mut self, event: &comms::event::Event) {
        match event {
            comms::event::Event::UserMessage(user_message) => {
                self.messages.push(user_message.clone());
            }
            _ => {}
        }
    }

    fn increment_timer(&mut self) {
        self.timer += 1;
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
        self.input.insert(self.cursor_position, new_char);

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
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    async fn submit_message(&mut self) {
        // TODO: handle the promise
        let _ = self
            .client
            .send_command(&command::UserCommand::SendMessage(
                command::SendMessageCommand {
                    room: "general".to_string(),
                    content: self.input.clone(),
                },
            ))
            .await;

        self.input.clear();
        self.reset_cursor();
    }
}

pub(crate) async fn main_loop(
    mut interrupt_rx: broadcast::Receiver<Interrupted>,
    mut client: Client,
    app: Arc<RwLock<App>>,
) -> anyhow::Result<Interrupted> {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    let mut event_stream = client.event_stream();

    let result = loop {
        tokio::select! {
            // Tick to terminate the select every N milliseconds
            _ = ticker.tick() => {
                let mut app = app.write().await;

                app.increment_timer();
            },
            Ok(event) = event_stream.recv() => {
                let mut app = app.write().await;

                app.handle_server_event(&event);
            }
            // Catch and handle interrupt signal to gracefully shutdown
            Ok(interrupted) = interrupt_rx.recv() => {
                break interrupted;
            }
        }
    };

    Ok(result)
}
