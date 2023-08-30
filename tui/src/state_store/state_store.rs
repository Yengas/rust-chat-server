use std::time::Duration;

use anyhow::Context;
use comms::{
    command,
    transport::{
        self,
        client::{CommandWriter, EventStream},
    },
};
use tokio::{
    net::TcpStream,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
};
use tokio_stream::StreamExt;

use crate::{Interrupted, Terminator};

use super::{action::Action, State};

pub struct StateStore {
    state_tx: UnboundedSender<State>,
}

impl StateStore {
    pub fn new() -> (Self, UnboundedReceiver<State>) {
        let (state_tx, state_rx) = mpsc::unbounded_channel::<State>();

        (StateStore { state_tx }, state_rx)
    }
}

type ServerHandle = (EventStream, CommandWriter);

async fn create_server_handle(addr: &str) -> anyhow::Result<ServerHandle> {
    let stream = TcpStream::connect(addr).await?;
    let (event_stream, command_writer) = transport::client::split_tcp_stream(stream);

    Ok((event_stream, command_writer))
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
            if let Some((event_stream, command_writer)) = opt_server_handle.as_mut() {
                tokio::select! {
                    // Handle the server events as they come in
                    maybe_event = event_stream.next() => match maybe_event {
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
                            if let Some(false) = state.try_set_active_room(room.as_str()).map(|room_data| room_data.has_joined) {
                                command_writer
                                    .write(&command::UserCommand::JoinRoom(command::JoinRoomCommand {
                                        room,
                                    }))
                                    .await
                                    .context("could not join room")?;
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
                        state.tick_timer();
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
                            state.mark_connection_request_start();
                            // emit event to re-render any part depending on the connection status
                            self.state_tx.send(state.clone())?;

                            match create_server_handle(&addr).await {
                                Ok(server_handle) => {
                                    // set the server handle and change status for further processing
                                    let _ = opt_server_handle.insert(server_handle);
                                    state.process_connection_request_result(Ok(addr));
                                    // ticker needs to be resetted to avoid showing time spent inputting and connecting to the server address
                                    ticker.reset();
                                },
                                Err(err) => {
                                    state.process_connection_request_result(Err(err));
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
