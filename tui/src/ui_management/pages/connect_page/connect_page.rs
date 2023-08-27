use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, State};

use crate::ui_management::framework::component::{Component, ComponentRender};

/// ConnectPage handles the connection to the server
pub struct ConnectPage {
    /// Action sender
    pub action_tx: UnboundedSender<Action>,
}

const SERVER_ADDR: &str = "localhost:8080";

impl Component for ConnectPage {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self
    where
        Self: Sized,
    {
        ConnectPage {
            action_tx: action_tx.clone(),
        }
        .move_with_state(state)
    }

    fn move_with_state(self, _state: &State) -> Self
    where
        Self: Sized,
    {
        ConnectPage { ..self }
    }

    fn name(&self) -> &str {
        "Connect Page"
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                let _ = self.action_tx.send(Action::ConnectToServerRequest {
                    addr: String::from(SERVER_ADDR),
                });
            }
            KeyCode::Char('q') => {
                let _ = self.action_tx.send(Action::Exit);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = self.action_tx.send(Action::Exit);
            }
            _ => {}
        }
    }
}

impl ComponentRender<()> for ConnectPage {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, _props: ()) {
        let [_, vertical_centered, _] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Min(1),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(frame.size())
        else {
            panic!("The main layout should have 3 chunks")
        };

        let [_, both_centered, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Min(1),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(vertical_centered)
        else {
            panic!("The horizontal layout should have 3 chunks")
        };

        let [container_addr_input, container_help_text] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
            .split(both_centered)
        else {
            panic!("The left layout should have 2 chunks")
        };

        let addr_input = Paragraph::new(Text::from(SERVER_ADDR)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Server Host and Port"),
        );
        frame.render_widget(addr_input, container_addr_input);

        let help_text = Paragraph::new(Text::from(Line::from(vec![
            "Press ".into(),
            "<Enter>".bold(),
            " to connect.".into(),
        ])));
        frame.render_widget(help_text, container_help_text);
    }
}
