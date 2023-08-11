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
        ChatRoomMetadata::new("general", "General discussions and community bonding"),
        ChatRoomMetadata::new("rust", "Talk about the Rust programming language"),
        ChatRoomMetadata::new("web-dev", "All about web development"),
        ChatRoomMetadata::new("ml", "Machine learning algorithms and research"),
        ChatRoomMetadata::new("tech-news", "Latest tech news and opinions"),
        ChatRoomMetadata::new("gaming", "Discuss games and gaming hardware"),
        ChatRoomMetadata::new("open-src", "Open source collaboration and projects"),
        ChatRoomMetadata::new("blockchain", "Blockchain and cryptocurrencies"),
        ChatRoomMetadata::new("startups", "Startup ideas and entrepreneurship"),
        ChatRoomMetadata::new("design", "Design principles and user experience"),
        ChatRoomMetadata::new("cloud-devops", "Cloud computing and DevOps practices"),
        ChatRoomMetadata::new("security", "Cybersecurity and ethical hacking"),
        ChatRoomMetadata::new("freelance", "Freelancing experiences and networking"),
        ChatRoomMetadata::new("hardware", "Hardware development and IoT"),
        ChatRoomMetadata::new("ai", "Discuss artificial intelligence topics"),
        ChatRoomMetadata::new("mobile-dev", "Mobile app development and tools"),
        ChatRoomMetadata::new("data-sci", "Data science techniques and tools"),
        ChatRoomMetadata::new("networking", "Networking protocols and technologies"),
        ChatRoomMetadata::new("os-dev", "Operating system development and kernel hacking"),
        ChatRoomMetadata::new("databases", "Database management and SQL"),
        ChatRoomMetadata::new("frontend", "Frontend development and frameworks"),
        ChatRoomMetadata::new("robotics", "Robotics engineering and automation"),
        ChatRoomMetadata::new("academia", "Research, papers, and academic discussions"),
        ChatRoomMetadata::new("career-advice", "Career growth and job-hunting tips"),
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
