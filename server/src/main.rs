use std::sync::Arc;

use anyhow::Context;
use tokio::{
    net::TcpListener,
    signal::unix::{signal, SignalKind},
    sync::{broadcast, Mutex},
    task::JoinSet,
};

use crate::room::{ChatRoom, ChatRoomMetadata};

mod room;
mod session;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() {
    let mut join_set: JoinSet<anyhow::Result<()>> = JoinSet::new();
    let chat_rooms: Vec<(ChatRoomMetadata, Arc<Mutex<ChatRoom>>)> = vec![
        ChatRoomMetadata::new(
            "general",
            "talking about topics which do not fall into any other room",
        ),
        ChatRoomMetadata::new("rust", "talking about the Rust programming language"),
    ]
    .into_iter()
    .map(|metadata| {
        let room = ChatRoom::new(&metadata);

        (metadata, Arc::new(Mutex::new(room)))
    })
    .collect();

    let mut interrupt =
        signal(SignalKind::interrupt()).expect("failed to create interrupt signal stream");
    let server = TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .expect("could not bind to the port");
    let (quit_tx, quit_rx) = broadcast::channel::<()>(1);

    println!("Listening on port {}", PORT);
    loop {
        tokio::select! {
            _ = interrupt.recv() => {
                println!("Server interrupted. Gracefully shutting down.");
                quit_tx.send(()).context("failed to send quit signal").unwrap();
                break;
            }
            Ok((socket, _)) = server.accept() => {
                join_set.spawn(session::handle_user_session(chat_rooms.clone(), quit_rx.resubscribe(), socket));
            }
        }
    }

    while join_set.join_next().await.is_some() {}
    println!("Server shut down");
}
