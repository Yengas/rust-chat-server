use anyhow::Context;
use comms::event;
use tokio::sync::broadcast;

#[derive(Debug)]
/// MessageSender is a struct that allows sending messages associated to a specific room
/// and a username. When a user joins a room, a MessageSender is created for that user.
pub struct MessageSender {
    broacast_tx: broadcast::Sender<event::Event>,
    room: String,
    session_id: String,
    username: String,
}

impl MessageSender {
    pub(super) fn new(
        broadcast_tx: broadcast::Sender<event::Event>,
        room: String,
        session_id: String,
        username: String,
    ) -> Self {
        MessageSender {
            broacast_tx: broadcast_tx,
            room,
            session_id,
            username,
        }
    }

    pub(super) fn session_id(&self) -> &str {
        &self.session_id
    }

    pub(super) fn username(&self) -> &str {
        &self.username
    }

    /// Send a message to the room
    pub fn send(&self, content: String) -> anyhow::Result<()> {
        self.broacast_tx
            .send(comms::event::Event::UserMessage(
                event::UserMessageBroadcastEvent {
                    room: self.room.clone(),
                    username: self.username.clone(),
                    content,
                },
            ))
            .context("could not write to the broadcast channel")?;

        Ok(())
    }
}
