use async_trait::async_trait;
use crossterm::event::KeyEvent;

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

#[async_trait(?Send)]
pub(super) trait WidgetHandler {
    fn activate(&mut self);
    fn deactivate(&mut self);
    fn name(&self) -> &str;
    async fn handle_key_event(&mut self, key: KeyEvent) -> WidgetKeyHandled;
    fn usage(&self) -> WidgetUsage;
}

#[derive(Debug, Clone)]
pub(super) enum WidgetKeyHandled {
    /// No further action needed
    Ok,
    /// Widget needs to lose focus
    LoseFocus,
}
