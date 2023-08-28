use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use comms::{
    command::UserCommand,
    event::{self, Event},
};
use tokio::{
    sync::{mpsc, Mutex},
    task::{AbortHandle, JoinSet},
};

use crate::room::{ChatRoom, MessageSender};

pub(super) struct ChatRoomManager {
    session_id: String,
    username: String,
    all_rooms: HashMap<String, Arc<Mutex<ChatRoom>>>,
    joined_rooms: HashMap<String, (MessageSender, AbortHandle)>,
    join_set: JoinSet<()>,
    mpsc_tx: mpsc::Sender<Event>,
    mpsc_rx: mpsc::Receiver<Event>,
}

impl ChatRoomManager {
    pub fn new(
        session_id: &str,
        username: &str,
        chat_rooms: HashMap<String, Arc<Mutex<ChatRoom>>>,
    ) -> Self {
        let (mpsc_tx, mpsc_rx) = mpsc::channel(100);

        ChatRoomManager {
            session_id: String::from(session_id),
            username: String::from(username),
            all_rooms: chat_rooms,
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
                    return Err(anyhow::anyhow!("already joined room"));
                }

                let room = self
                    .all_rooms
                    .get(&cmd.room)
                    .ok_or_else(|| anyhow::anyhow!("room not found"))?;

                let (urp, participants) = {
                    let mut room = room.lock().await;
                    let urp = room.add_participant(self.session_id.clone(), self.username.clone());

                    (urp, room.participants().clone())
                };

                let (message_sender, mut broadcast_rx) = (urp.message_sender, urp.broadcast_rx);

                // spawn a task to forward broadcasted messages to the users' mpsc channel
                // hence the user can receive messages from different rooms via single channel
                let abort_handle = self.join_set.spawn({
                    let mpsc_tx = self.mpsc_tx.clone();

                    // start with sending the user joined room event as a reply to the user
                    mpsc_tx
                        .send(Event::UserJoinedRoom(event::UserJoinedRoomReplyEvent {
                            room: cmd.room.clone(),
                            users: participants.into_iter().collect(),
                        }))
                        .await?;

                    async move {
                        while let Ok(event) = broadcast_rx.recv().await {
                            let _ = mpsc_tx.send(event).await;
                        }
                    }
                });

                // store references to the message sender and abort handle
                // this is used to send messages to the room and to cancel the task when user leaves the room
                self.joined_rooms
                    .insert(cmd.room.clone(), (message_sender, abort_handle));
            }
            UserCommand::SendMessage(cmd) => {
                if let Some((message_sender, _)) = self.joined_rooms.get(&cmd.room) {
                    let _ = message_sender.send(cmd.content);
                }
            }
            UserCommand::LeaveRoom(cmd) => {
                // remove the room from joined rooms and trigger cleanup for the removed values
                if let Some(urp) = self.joined_rooms.remove(&cmd.room) {
                    self.cleanup_room(&cmd.room, urp).await?;
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

        for (room_name, urp) in drained {
            self.cleanup_room(&room_name, urp).await?;
        }

        Ok(())
    }

    /// Cleanup the room by removing the user from the room and
    /// aborting the task that forwards broadcasted messages to the user
    async fn cleanup_room(
        &mut self,
        room_name: &String,
        (message_sender, abort_handle): (MessageSender, AbortHandle),
    ) -> anyhow::Result<()> {
        {
            let mut room = self
                .all_rooms
                .get(room_name)
                .ok_or_else(|| anyhow::anyhow!("room not found"))?
                .lock()
                .await;

            room.remove_participant(message_sender);
        }

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
