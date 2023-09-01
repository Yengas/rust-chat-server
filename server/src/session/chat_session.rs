use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use comms::{
    command::UserCommand,
    event::{self, Event},
};
use tokio::{
    sync::mpsc,
    task::{AbortHandle, JoinSet},
};

use crate::room_manager::{RoomManager, SessionAndUserId, UserSessionHandle};

pub(super) struct ChatSession {
    session_and_user_id: SessionAndUserId,
    room_manager: Arc<RoomManager>,
    joined_rooms: HashMap<String, (UserSessionHandle, AbortHandle)>,
    join_set: JoinSet<()>,
    mpsc_tx: mpsc::Sender<Event>,
    mpsc_rx: mpsc::Receiver<Event>,
}

impl ChatSession {
    pub fn new(session_id: &str, user_id: &str, room_manager: Arc<RoomManager>) -> Self {
        let (mpsc_tx, mpsc_rx) = mpsc::channel(100);
        let session_and_user_id = SessionAndUserId {
            session_id: String::from(session_id),
            user_id: String::from(user_id),
        };

        ChatSession {
            session_and_user_id,
            room_manager,
            joined_rooms: HashMap::new(),
            join_set: JoinSet::new(),
            mpsc_tx,
            mpsc_rx,
        }
    }

    /// Handle a user command related to room management such as; join, leave, send message
    pub async fn handle_user_command(&mut self, cmd: UserCommand) -> anyhow::Result<()> {
        match cmd {
            UserCommand::JoinRoom(cmd) => {
                if self.joined_rooms.contains_key(&cmd.room) {
                    return Err(anyhow::anyhow!("already joined room '{}'", &cmd.room));
                }

                let (mut broadcast_rx, user_session_handle, user_ids) = self
                    .room_manager
                    .join_room(&cmd.room, &self.session_and_user_id)
                    .await?;

                // spawn a task to forward broadcasted messages to the users' mpsc channel
                // hence the user can receive messages from different rooms via single channel
                let abort_handle = self.join_set.spawn({
                    let mpsc_tx = self.mpsc_tx.clone();

                    // start with sending the user joined room event as a reply to the user
                    mpsc_tx
                        .send(Event::UserJoinedRoom(event::UserJoinedRoomReplyEvent {
                            room: cmd.room.clone(),
                            users: user_ids,
                        }))
                        .await?;

                    async move {
                        while let Ok(event) = broadcast_rx.recv().await {
                            let _ = mpsc_tx.send(event).await;
                        }
                    }
                });

                // store references to the user session handle and abort handle
                // this is used to send messages to the room and to cancel the task when user leaves the room
                self.joined_rooms
                    .insert(cmd.room.clone(), (user_session_handle, abort_handle));
            }
            UserCommand::SendMessage(cmd) => {
                if let Some((user_session_handle, _)) = self.joined_rooms.get(&cmd.room) {
                    let _ = user_session_handle.send_message(cmd.content);
                }
            }
            UserCommand::LeaveRoom(cmd) => {
                // remove the room from joined rooms and drop user session handle for the room
                if let Some(urp) = self.joined_rooms.remove(&cmd.room) {
                    self.cleanup_room(urp).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    // TODO: optimize the performance of this function. leaving one by one may not be a good idea.
    /// Leave all the rooms the user is currently participating in
    pub async fn leave_all_rooms(&mut self) -> anyhow::Result<()> {
        // drain the joined rooms to a variable, necessary to avoid borrowing self
        let drained = self.joined_rooms.drain().collect::<Vec<_>>();

        for (_, urp) in drained {
            self.cleanup_room(urp).await?;
        }

        Ok(())
    }

    /// Cleanup the room by removing the user from the room and
    /// aborting the task that forwards broadcasted messages to the user
    async fn cleanup_room(
        &mut self,
        (user_session_handle, abort_handle): (UserSessionHandle, AbortHandle),
    ) -> anyhow::Result<()> {
        self.room_manager
            .drop_user_session_handle(user_session_handle)
            .await?;

        abort_handle.abort();

        Ok(())
    }

    /// Recieve an event that may have originated from any of the rooms the user is actively participating in
    pub async fn recv(&mut self) -> anyhow::Result<Event> {
        self.mpsc_rx
            .recv()
            .await
            .context("could not recv from the broadcast channel")
    }
}
