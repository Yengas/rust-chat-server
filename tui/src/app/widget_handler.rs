use async_trait::async_trait;
use crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub(crate) struct WidgetUsageKey {
    pub(crate) keys: Vec<String>,
    pub(crate) description: String,
}

#[derive(Debug, Clone)]
pub(crate) struct WidgetUsage {
    pub(crate) description: Option<String>,
    pub(crate) keys: Vec<WidgetUsageKey>,
}

#[async_trait(?Send)]
pub(super) trait WidgetHandler {
    fn activate(&mut self);
    fn deactivate(&mut self);
    fn name(&self) -> &str;
    fn usage(&self) -> WidgetUsage;
    async fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled;
}

#[derive(Debug, Clone)]
pub(super) enum WidgetKeyHandled {
    /// No further action needed
    Ok,
    /// Widget needs to lose focus
    LoseFocus,
}
