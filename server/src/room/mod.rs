use comms::event;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct MessageSender {
    broacast_tx: broadcast::Sender<event::Event>,
    room: String,
    username: String,
}

impl MessageSender {
    fn new(broadcast_tx: broadcast::Sender<event::Event>, room: String, username: String) -> Self {
        MessageSender {
            broacast_tx: broadcast_tx,
            room,
            username,
        }
    }

    pub fn send(&self, content: String) -> anyhow::Result<()> {
        self.broacast_tx
            .send(comms::event::Event::UserMessage(event::UserMessageEvent {
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
    pub broadcast_rx: broadcast::Receiver<event::Event>,
}

impl UserRoomParticipation {
    fn new(message_sender: MessageSender, broadcast_rx: broadcast::Receiver<event::Event>) -> Self {
        UserRoomParticipation {
            message_sender,
            broadcast_rx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatRoomMetadata {
    pub name: String,
    pub description: String,
}

impl ChatRoomMetadata {
    pub fn new(name: &str, description: &str) -> Self {
        ChatRoomMetadata {
            name: String::from(name),
            description: String::from(description),
        }
    }
}

#[derive(Debug)]
pub struct ChatRoom {
    name: String,
    participants: Vec<String>,
    broadcast_tx: broadcast::Sender<event::Event>,
}

impl ChatRoom {
    pub fn new(metadata: &ChatRoomMetadata) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);

        ChatRoom {
            name: String::from(&metadata.name),
            broadcast_tx,
            participants: Vec::new(),
        }
    }

    pub fn add_participant(&mut self, username: String) -> UserRoomParticipation {
        self.participants.push(username.clone());

        let broadcast_tx = self.broadcast_tx.clone();
        let broadcast_rx = broadcast_tx.subscribe();
        let message_sender = MessageSender::new(broadcast_tx, self.name.clone(), username.clone());

        let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
            event::RoomParticipationEvent {
                username,
                room: self.name.clone(),
                status: event::RoomParticipationStatus::Joined,
            },
        ));

        UserRoomParticipation::new(message_sender, broadcast_rx)
    }

    pub fn remove_participant(&mut self, username: &String) {
        self.participants.retain(|p| p != username);

        let _ = self.broadcast_tx.send(event::Event::RoomParticipation(
            event::RoomParticipationEvent {
                username: username.clone(),
                room: self.name.clone(),
                status: event::RoomParticipationStatus::Left,
            },
        ));
    }
}
