use anyhow::Context;
use comms::{
    command::UserCommand,
    event::{Event, RoomParticipationEvent},
};
use tokio::{
    net::TcpListener,
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    task::JoinSet,
};

mod session;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() {
    let mut join_set: JoinSet<()> = JoinSet::new();

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
                join_set.spawn(session::handle_user_session(quit_rx.resubscribe(), socket));
            }
        }
    }

    while join_set.join_next().await.is_some() {}
    println!("Server shut down");
}
