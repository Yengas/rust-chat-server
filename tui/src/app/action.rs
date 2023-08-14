#[derive(Debug, Clone)]
pub enum Action {
    SendMessage { content: String },
    SelectRoom { room: String },
    Exit,
}
