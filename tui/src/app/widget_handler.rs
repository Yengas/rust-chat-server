use async_trait::async_trait;
use crossterm::event::KeyEvent;

#[async_trait(?Send)]
pub(super) trait WidgetHandler {
    fn activate(&mut self);
    fn deactivate(&mut self);
    async fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled;
}

#[derive(Debug, Clone)]
pub(super) enum WidgetKeyHandled {
    /// No further action needed
    Ok,
    /// Widget needs to lose focus
    LoseFocus,
}
