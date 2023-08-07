use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use comms::{command::UserCommand, event::Event};
use tokio::{
    sync::{mpsc, Mutex},
    task::{AbortHandle, JoinSet},
};

use crate::room::{ChatRoom, MessageSender, UserRoomParticipation};

pub(super) struct ChatRoomManager {
    username: String,
    all_rooms: HashMap<String, Arc<Mutex<ChatRoom>>>,
    joined_rooms: HashMap<String, (MessageSender, AbortHandle)>,
    join_set: JoinSet<()>,
    mpsc_tx: mpsc::Sender<Event>,
    mpsc_rx: mpsc::Receiver<Event>,
}

impl ChatRoomManager {
    pub fn new(username: &str, chat_rooms: HashMap<String, Arc<Mutex<ChatRoom>>>) -> Self {
        let (mpsc_tx, mpsc_rx) = mpsc::channel(100);

        ChatRoomManager {
            username: String::from(username),
            all_rooms: chat_rooms,
            joined_rooms: HashMap::new(),
            join_set: JoinSet::new(),
            mpsc_tx,
            mpsc_rx,
        }
    }

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

                let urp: UserRoomParticipation = {
                    let mut room = room.lock().await;

                    room.add_participant(self.username.clone())
                };

                let (message_sender, mut broadcast_rx) = (urp.message_sender, urp.broadcast_rx);
                let abort_handle = self.join_set.spawn({
                    let mpsc_tx = self.mpsc_tx.clone();

                    async move {
                        while let Ok(event) = broadcast_rx.recv().await {
                            let _ = mpsc_tx.send(event).await;
                        }
                    }
                });

                self.joined_rooms
                    .insert(cmd.room.clone(), (message_sender, abort_handle));
            }
            UserCommand::SendMessage(cmd) => {
                if let Some((message_sender, _)) = self.joined_rooms.get(&cmd.room) {
                    let _ = message_sender.send(cmd.content);
                }
            }
            UserCommand::LeaveRoom(cmd) => {
                if let Some(urp) = self.joined_rooms.remove(&cmd.room) {
                    self.cleanup_room(&cmd.room, urp).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    // TODO: optimize the performance of this function. leaving one by one may not be a good idea.
    pub async fn leave_all_rooms(&mut self) -> anyhow::Result<()> {
        let drained = self.joined_rooms.drain().collect::<Vec<_>>();
        for (room_name, urp) in drained {
            self.cleanup_room(&room_name, urp).await?;
        }

        Ok(())
    }

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

            room.remove_participant(&self.username);
        }

        abort_handle.abort();
        drop(message_sender);

        Ok(())
    }

    pub async fn recv(&mut self) -> anyhow::Result<Event> {
        self.mpsc_rx
            .recv()
            .await
            .context("could not recv from the broadcast channel")
    }
}
