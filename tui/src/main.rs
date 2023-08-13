use app::AppHolder;
use manager::Manager;
use termination::create_termination;

pub(self) mod app;
mod manager;
mod termination;

pub(self) use termination::{Interrupted, Terminator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (terminator, mut interrupt_rx) = create_termination();
    let (app_holder, state_rx) = AppHolder::new();
    let (manager, action_rx) = Manager::new();

    tokio::try_join!(
        app_holder.main_loop(terminator.clone(), action_rx, interrupt_rx.resubscribe()),
        manager.main_loop(terminator, state_rx, interrupt_rx.resubscribe()),
    )?;

    if let Ok(reason) = interrupt_rx.recv().await {
        match reason {
            Interrupted::UserInt => println!("exited per user request"),
            Interrupted::OsSigInt => println!("exited because of an os sig int"),
            Interrupted::ServerDisconnected => {
                println!("exited because remote server has disconnected")
            }
        }
    } else {
        println!("exited because of an unexpected error");
    }

    Ok(())
}
