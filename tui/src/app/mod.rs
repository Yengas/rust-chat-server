use std::time::Duration;

use anyhow::Context;
use comms::command;
use tokio::{
    net::TcpStream,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
};
use tokio_stream::StreamExt;

use crate::{Interrupted, Terminator};

use self::action::Action;
pub use self::state::{MessageBoxItem, RoomData, State};

pub mod action;
mod client;
mod state;

pub struct AppHolder {
    state_tx: UnboundedSender<State>,
}

impl AppHolder {
    pub fn new() -> (Self, UnboundedReceiver<State>) {
        let (state_tx, state_rx) = mpsc::unbounded_channel::<State>();

        (AppHolder { state_tx }, state_rx)
    }
}

impl AppHolder {
    pub async fn main_loop(
        self,
        mut terminator: Terminator,
        mut action_rx: UnboundedReceiver<Action>,
        mut interrupt_rx: broadcast::Receiver<Interrupted>,
    ) -> anyhow::Result<Interrupted> {
        let stream = TcpStream::connect("localhost:8080").await?;
        let (mut event_stream, mut command_writer) = client::split_stream(stream);
        let mut state = State::new();

        // the initial state once

        let mut ticker = tokio::time::interval(Duration::from_secs(1));

        let result = loop {
            tokio::select! {
                // Handle the server events as they come in
                maybe_event = event_stream.next() => match maybe_event {
                    Some(Ok(event)) => {
                        state.handle_server_event(&event);
                    },
                    None => {
                        let _ = terminator.terminate(Interrupted::ServerDisconnected);

                        break Interrupted::ServerDisconnected;
                    },
                    _ => (),
                },
                // Handle the actions coming from the UI
                Some(action) = action_rx.recv() => match action {
                    Action::SendMessage { content } => {
                        if let Some(active_room) = state.active_room.as_ref() {
                            command_writer
                                .write(&command::UserCommand::SendMessage(
                                    command::SendMessageCommand {
                                        room: active_room.clone(),
                                        content,
                                    },
                                ))
                                .await
                                .context("could not send message")?;
                        }
                    },
                    Action::SelectRoom { room } => {
                        if let Some(room_data) = state.room_data_map.get_mut(room.as_str()) {
                            state.active_room = Some(room.clone());

                            if !room_data.has_joined {
                                command_writer
                                    .write(&command::UserCommand::JoinRoom(command::JoinRoomCommand {
                                        room,
                                    }))
                                    .await
                                    .context("could not join room")?;
                            }
                        }
                    },
                    Action::Exit => {
                        let _ = terminator.terminate(Interrupted::UserInt);

                        break Interrupted::UserInt;
                    }
                },
                // Tick to terminate the select every N milliseconds
                _ = ticker.tick() => {
                    state.timer += 1;
                },
                // Catch and handle interrupt signal to gracefully shutdown
                Ok(interrupted) = interrupt_rx.recv() => {
                    break interrupted;
                }
            }

            self.state_tx.send(state.clone())?;
        };

        Ok(result)
    }
}
