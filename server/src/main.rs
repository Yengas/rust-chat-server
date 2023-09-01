use std::sync::Arc;

use anyhow::Context;
use room_manager::RoomManagerBuilder;
use tokio::{
    net::TcpListener,
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    task::JoinSet,
};

mod room_manager;
mod session;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() {
    let mut join_set: JoinSet<anyhow::Result<()>> = JoinSet::new();
    let room_manager = Arc::new(
        RoomManagerBuilder::new()
            .create_room("general", "General discussions and community bonding")
            .create_room("rust", "Talk about the Rust programming language")
            .create_room("web-dev", "All about web development")
            .create_room("ml", "Machine learning algorithms and research")
            .create_room("tech-news", "Latest tech news and opinions")
            .create_room("gaming", "Discuss games and gaming hardware")
            .create_room("open-src", "Open source collaboration and projects")
            .create_room("blockchain", "Blockchain and cryptocurrencies")
            .create_room("startups", "Startup ideas and entrepreneurship")
            .create_room("design", "Design principles and user experience")
            .create_room("cloud-devops", "Cloud computing and DevOps practices")
            .create_room("security", "Cybersecurity and ethical hacking")
            .create_room("freelance", "Freelancing experiences and networking")
            .create_room("hardware", "Hardware development and IoT")
            .create_room("ai", "Discuss artificial intelligence topics")
            .create_room("mobile-dev", "Mobile app development and tools")
            .create_room("data-sci", "Data science techniques and tools")
            .create_room("networking", "Networking protocols and technologies")
            .create_room("os-dev", "Operating system development and kernel hacking")
            .create_room("databases", "Database management and SQL")
            .create_room("frontend", "Frontend development and frameworks")
            .create_room("robotics", "Robotics engineering and automation")
            .create_room("academia", "Research, papers, and academic discussions")
            .create_room("career-advice", "Career growth and job-hunting tips")
            .build(),
    );

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
                join_set.spawn(session::handle_user_session(Arc::clone(&room_manager), quit_rx.resubscribe(), socket));
            }
        }
    }

    while join_set.join_next().await.is_some() {}
    println!("Server shut down");
}
