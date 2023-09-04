use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::ServerConnectionStatus;
use crate::state_store::{action::Action, State};

use crate::ui_management::components::input_box;
use crate::ui_management::components::{input_box::InputBox, Component, ComponentRender};

struct Props {
    error_message: Option<String>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Props {
            error_message: if let ServerConnectionStatus::Errored { err } =
                &state.server_connection_status
            {
                Some(err.to_string())
            } else {
                None
            },
        }
    }
}

/// ConnectPage handles the connection to the server
pub struct ConnectPage {
    /// Action sender
    pub action_tx: UnboundedSender<Action>,
    // Mapped Props from State
    props: Props,
    // Internal Components
    input_box: InputBox,
}

impl ConnectPage {
    fn connect_to_server(&mut self) {
        if self.input_box.is_empty() {
            return;
        }

        let _ = self.action_tx.send(Action::ConnectToServerRequest {
            addr: self.input_box.text().to_string(),
        });
    }
}

const DEFAULT_SERVER_ADDR: &str = "localhost:8080";

impl Component for ConnectPage {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self
    where
        Self: Sized,
    {
        let mut input_box = InputBox::new(state, action_tx.clone());
        input_box.set_text(DEFAULT_SERVER_ADDR);

        ConnectPage {
            action_tx: action_tx.clone(),
            //
            props: Props::from(state),
            //
            input_box,
        }
        .move_with_state(state)
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        ConnectPage {
            props: Props::from(state),
            ..self
        }
    }

    fn name(&self) -> &str {
        "Connect Page"
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        self.input_box.handle_key_event(key);

        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Enter => {
                self.connect_to_server();
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

        let [container_addr_input, container_help_text, container_error_message] =
            *Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(both_centered)
        else {
            panic!("The left layout should have 3 chunks")
        };

        self.input_box.render(
            frame,
            input_box::RenderProps {
                title: "Server Host and Port".into(),
                area: container_addr_input,
                border_color: Color::Yellow,
                show_cursor: true,
            },
        );

        let help_text = Paragraph::new(Text::from(Line::from(vec![
            "Press ".into(),
            "<Enter>".bold(),
            " to connect".into(),
        ])));
        frame.render_widget(help_text, container_help_text);

        let error_message = Paragraph::new(if let Some(err) = self.props.error_message.as_ref() {
            Text::from(format!("Error: {}", err.as_str()))
        } else {
            Text::from("")
        })
        .wrap(Wrap { trim: true })
        .style(
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::SLOW_BLINK | Modifier::ITALIC),
        );

        frame.render_widget(error_message, container_error_message);
    }
}
