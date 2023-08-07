use comms::event::{Event, UserMessageEvent};
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct MessageSender {
    broacast_tx: broadcast::Sender<Event>,
    room: String,
    username: String,
}

impl MessageSender {
    fn new(broadcast_tx: broadcast::Sender<Event>, room: String, username: String) -> Self {
        MessageSender {
            broacast_tx: broadcast_tx,
            room,
            username,
        }
    }

    pub fn send(&self, content: String) -> anyhow::Result<()> {
        self.broacast_tx
            .send(comms::event::Event::UserMessage(UserMessageEvent {
                room: self.room.clone(),
                username: self.username.clone(),
                content,
            }))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct UserRoomParticipation {
    pub message_sender: MessageSender,
    pub broadcast_rx: broadcast::Receiver<Event>,
}

impl UserRoomParticipation {
    fn new(message_sender: MessageSender, broadcast_rx: broadcast::Receiver<Event>) -> Self {
        UserRoomParticipation {
            message_sender,
            broadcast_rx,
        }
    }
}

pub struct ChatRoom {
    name: String,
    description: String,
    participants: Vec<String>,
    broadcast_tx: broadcast::Sender<Event>,
}

impl ChatRoom {
    pub fn new(name: &str, description: &str) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);

        ChatRoom {
            name: String::from(name),
            description: String::from(description),
            broadcast_tx,
            participants: Vec::new(),
        }
    }

    pub fn add_participant(&mut self, username: String) -> UserRoomParticipation {
        self.participants.push(username.clone());

        let broadcast_tx = self.broadcast_tx.clone();
        let broadcast_rx = broadcast_tx.subscribe();
        let message_sender = MessageSender::new(broadcast_tx, self.name.clone(), username);

        UserRoomParticipation::new(message_sender, broadcast_rx)
    }

    pub fn remove_participant(&mut self, username: &String) {
        self.participants.retain(|p| p != username);
    }
}
