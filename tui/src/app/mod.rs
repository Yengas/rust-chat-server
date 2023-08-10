use std::{sync::Arc, time::Duration};

use comms::event;
use tokio::{
    net::TcpStream,
    sync::{broadcast, RwLock},
};
use tokio_stream::StreamExt;

use crate::{Interrupted, Terminator};

pub(crate) use widget_handler::WidgetUsage;

use self::{app::App, client::BoxedStream};

pub mod app;
mod client;
mod input_box;
mod room_list;
mod shared_state;
mod widget_handler;

pub struct AppHolder {
    app: Arc<RwLock<App>>,
    event_stream: BoxedStream<anyhow::Result<event::Event>>,
}

pub async fn create_app_holder(terminator: Terminator) -> anyhow::Result<AppHolder> {
    let stream = TcpStream::connect("localhost:8080").await?;
    let (event_stream, command_writer) = client::split_stream(stream);

    Ok(AppHolder {
        app: Arc::new(RwLock::new(App::new(command_writer, terminator))),
        event_stream,
    })
}

impl AppHolder {
    pub fn take_app_reference(&self) -> Arc<RwLock<App>> {
        Arc::clone(&self.app)
    }

    pub async fn main_loop(
        &mut self,
        mut interrupt_rx: broadcast::Receiver<Interrupted>,
    ) -> anyhow::Result<Interrupted> {
        let mut ticker = tokio::time::interval(Duration::from_secs(1));

        let result = loop {
            tokio::select! {
                Some(Ok(event)) = self.event_stream.next() => {
                    let mut app = self.app.write().await;

                    app.handle_server_event(&event);
                }
                // Tick to terminate the select every N milliseconds
                _ = ticker.tick() => {
                    let mut app = self.app.write().await;

                    app.increment_timer();
                },
                // Catch and handle interrupt signal to gracefully shutdown
                Ok(interrupted) = interrupt_rx.recv() => {
                    break interrupted;
                }
            }
        };

        Ok(result)
    }
}
