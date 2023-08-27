use std::time::Duration;

use anyhow::Context;
use comms::{command, event::Event};
use tokio::{
    net::{tcp::OwnedWriteHalf, TcpStream},
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
};
use tokio_stream::StreamExt;

use crate::{state_store::ServerConnectionStatus, Interrupted, Terminator};

use super::{
    action::Action,
    client::{self, BoxedStream, CommandWriter},
    State,
};

pub struct StateStore {
    state_tx: UnboundedSender<State>,
}

impl StateStore {
    pub fn new() -> (Self, UnboundedReceiver<State>) {
        let (state_tx, state_rx) = mpsc::unbounded_channel::<State>();

        (StateStore { state_tx }, state_rx)
    }
}

struct ServerHandle {
    event_stream: BoxedStream<anyhow::Result<Event>>,
    command_writer: CommandWriter<OwnedWriteHalf>,
}

async fn create_server_handle(addr: &str) -> anyhow::Result<ServerHandle> {
    let stream = TcpStream::connect(addr).await?;
    let (event_stream, command_writer) = client::split_stream(stream);

    Ok(ServerHandle {
        event_stream,
        command_writer,
    })
}

impl StateStore {
    pub async fn main_loop(
        self,
        mut terminator: Terminator,
        mut action_rx: UnboundedReceiver<Action>,
        mut interrupt_rx: broadcast::Receiver<Interrupted>,
    ) -> anyhow::Result<Interrupted> {
        let mut opt_server_handle: Option<ServerHandle> = None;
        let mut state = State::default();

        // the initial state once
        self.state_tx.send(state.clone())?;

        let mut ticker = tokio::time::interval(Duration::from_secs(1));

        let result = loop {
            if let Some(server_handle) = opt_server_handle.as_mut() {
                tokio::select! {
                    // Handle the server events as they come in
                    maybe_event = server_handle.event_stream.next() => match maybe_event {
                        Some(Ok(event)) => {
                            state.handle_server_event(&event);
                        },
                        // server disconnected, we need to reset the state
                        None => {
                            opt_server_handle = None;
                            state = State::default();
                        },
                        _ => (),
                    },
                    // Handle the actions coming from the UI
                    // and process them to do async operations
                    Some(action) = action_rx.recv() => match action {
                        Action::SendMessage { content } => {
                            if let Some(active_room) = state.active_room.as_ref() {
                                server_handle.command_writer
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
                                    server_handle.command_writer
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
                        },
                        _ => (),
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
            } else {
                tokio::select! {
                    Some(action) = action_rx.recv() => match action {
                        Action::ConnectToServerRequest { addr } => {
                            state.server_connection_status = ServerConnectionStatus::Connecting;
                            // emit event to re-render any part depending on the connection status
                            self.state_tx.send(state.clone())?;

                            match create_server_handle(&addr).await {
                                Ok(server_handle) => {
                                    // set the server handle and change status for further processing
                                    let _ = opt_server_handle.insert(server_handle);
                                    state.server_connection_status = ServerConnectionStatus::Connected { addr };
                                    // ticker needs to be resetted to avoid showing time spent inputting and connecting to the server address
                                    ticker.reset();
                                },
                                Err(err) => {
                                    state.server_connection_status = ServerConnectionStatus::Errored { err: err.to_string() };
                                }
                            }
                        },
                        Action::Exit => {
                            let _ = terminator.terminate(Interrupted::UserInt);

                            break Interrupted::UserInt;
                        },
                        _ => (),
                    },
                    // Catch and handle interrupt signal to gracefully shutdown
                    Ok(interrupted) = interrupt_rx.recv() => {
                        break interrupted;
                    }
                }
            }

            self.state_tx.send(state.clone())?;
        };

        Ok(result)
    }
}
