use comms::event::{Event, RoomParticipationEvent};
use tokio::{net::TcpStream, sync::broadcast};
use tokio_stream::StreamExt;

mod raw_socket;

pub async fn handle_user_session(mut quit_rx: broadcast::Receiver<()>, stream: TcpStream) -> () {
    let (mut commands, mut event_writer) = raw_socket::split_stream(stream);

    loop {
        tokio::select! {
            cmd = commands.next() => {
                if cmd.is_none() {
                    println!("Client disconnected.");
                    break;
                }

                println!("Received command: {:?}", cmd);

                event_writer.write(&Event::RoomParticipation(RoomParticipationEvent{
                    room: "test".to_string(),
                    username: "test".to_string(),
                    status: comms::event::RoomParticipationStatus::Joined,
                })).await.expect("failed to write event to socket");
            }
            Ok(_) = quit_rx.recv() => {
                drop(event_writer);
                println!("Gracefully shutting down user socket.");
                break;
            }
        }
    }
}
