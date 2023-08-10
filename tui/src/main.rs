use termination::create_termination;

pub(self) mod app;
mod manager;
mod termination;

pub(self) use termination::{Interrupted, Terminator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (terminator, interrupt_rx) = create_termination();
    let mut app_holder = app::create_app_holder(terminator.clone()).await?;

    tokio::try_join!(
        manager::main_loop(interrupt_rx.resubscribe(), app_holder.take_app_reference()),
        app_holder.main_loop(interrupt_rx),
    )?;

    Ok(())
}
