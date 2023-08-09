pub(crate) struct SharedState {
    pub(crate) active_room: Option<String>,
}

impl SharedState {
    pub(crate) fn new() -> Self {
        Self { active_room: None }
    }
}
