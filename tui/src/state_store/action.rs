#[derive(Debug, Clone)]
pub enum Action {
    ConnectToServerRequest { addr: String },
    SendMessage { content: String },
    SelectRoom { room: String },
    Exit,
}
