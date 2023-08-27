use crossterm::event::KeyEvent;
use ratatui::{prelude::Backend, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, ServerConnectionStatus, State};

use self::{chat_page::ChatPage, connect_page::ConnectPage};

use super::components::{Component, ComponentRender};

mod chat_page;
mod connect_page;

enum ActivePage {
    ChatPage,
    ConnectPage,
}

struct Props {
    active_page: ActivePage,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Props {
            active_page: match state.server_connection_status {
                ServerConnectionStatus::Connected { .. } => ActivePage::ChatPage,
                _ => ActivePage::ConnectPage,
            },
        }
    }
}

pub struct AppRouter {
    props: Props,
    //
    chat_page: ChatPage,
    connect_page: ConnectPage,
}

impl AppRouter {
    fn get_active_page_component(&self) -> &dyn Component {
        match self.props.active_page {
            ActivePage::ChatPage => &self.chat_page,
            ActivePage::ConnectPage => &self.connect_page,
        }
    }

    fn get_active_page_component_mut(&mut self) -> &mut dyn Component {
        match self.props.active_page {
            ActivePage::ChatPage => &mut self.chat_page,
            ActivePage::ConnectPage => &mut self.connect_page,
        }
    }
}

impl Component for AppRouter {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self
    where
        Self: Sized,
    {
        AppRouter {
            props: Props::from(state),
            //
            chat_page: ChatPage::new(state, action_tx.clone()),
            connect_page: ConnectPage::new(state, action_tx.clone()),
        }
        .move_with_state(state)
    }

    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        AppRouter {
            props: Props::from(state),
            //
            chat_page: self.chat_page.move_with_state(state),
            connect_page: self.connect_page.move_with_state(state),
        }
    }

    // route all functions to the active page
    fn name(&self) -> &str {
        self.get_active_page_component().name()
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        self.get_active_page_component_mut().handle_key_event(key)
    }
}

impl ComponentRender<()> for AppRouter {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: ()) {
        match self.props.active_page {
            ActivePage::ChatPage => self.chat_page.render(frame, props),
            ActivePage::ConnectPage => self.connect_page.render(frame, props),
        }
    }
}
