use crossterm::event::KeyEvent;
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, State};

#[derive(Debug, Clone)]
pub struct WidgetUsageKey {
    pub keys: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct WidgetUsage {
    pub description: Option<String>,
    pub keys: Vec<WidgetUsageKey>,
}

pub(super) trait WidgetHandler {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self
    where
        Self: Sized;
    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized;

    fn name(&self) -> &str;

    fn activate(&mut self);
    fn deactivate(&mut self);
    fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled;

    fn usage(&self) -> WidgetUsage;
}

#[derive(Debug, Clone)]
pub(super) enum WidgetKeyHandled {
    /// No further action needed
    Ok,
    /// Widget needs to lose focus
    LoseFocus,
}
