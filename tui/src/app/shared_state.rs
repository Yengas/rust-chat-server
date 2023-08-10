pub struct SharedState {
    pub active_room: Option<String>,
}

impl SharedState {
    pub fn new() -> Self {
        Self { active_room: None }
    }
}
