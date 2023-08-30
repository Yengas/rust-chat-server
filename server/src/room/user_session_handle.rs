use anyhow::Context;
use comms::event;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct SessionAndUsername {
    pub session_id: String,
    pub username: String,
}

#[derive(Debug)]
/// [UserSessionHandle] is a handle that allows a specific user/session pair to
/// send messages to a specific room.
///
/// It is created when a user joins a room and is handed out to the user.
pub struct UserSessionHandle {
    /// The name of the room which is associated with this handle
    room: String,
    /// The channel to use for sending events to the all users of the room
    broadcast_tx: broadcast::Sender<event::Event>,
    /// The session and username associated with this handle
    session_and_username: SessionAndUsername,
}

impl UserSessionHandle {
    pub(super) fn new(
        room: String,
        broadcast_tx: broadcast::Sender<event::Event>,
        session_and_username: SessionAndUsername,
    ) -> Self {
        UserSessionHandle {
            room,
            broadcast_tx,
            session_and_username,
        }
    }

    pub(super) fn session_id(&self) -> &str {
        &self.session_and_username.session_id
    }

    pub(super) fn username(&self) -> &str {
        &self.session_and_username.username
    }

    /// Send a message to the room
    pub fn send_message(&self, content: String) -> anyhow::Result<()> {
        self.broadcast_tx
            .send(comms::event::Event::UserMessage(
                event::UserMessageBroadcastEvent {
                    room: self.room.clone(),
                    username: self.session_and_username.username.clone(),
                    content,
                },
            ))
            .context("could not write to the broadcast channel")?;

        Ok(())
    }
}
