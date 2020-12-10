use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use std::ops::Deref;

mod toggle;

#[tokio::main]
async fn main() {
    let (system, mut state) = System::new();

    system.boop();
    system.stop();

    system.boop();
    system.stop();

    state.start().await;
    state.start().await;
}

enum Msg {
    Stop,
    Boop,
}

struct System(UnboundedSender<Msg>);

impl Deref for System {
    type Target = UnboundedSender<Msg>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl System {
    fn new() -> (System, State) {
        let (sender, receiver) = unbounded_channel();

        (System(sender), State(receiver))
    }

    fn stop(&self) {
        if let Err(_) = self.0.send(Msg::Stop) { }
    }

    fn boop(&self) {
        if let Err(_) = self.0.send(Msg::Boop) { }
    }
}

struct State(UnboundedReceiver<Msg>);

impl State {
    async fn start(&mut self) {
        while let Some(msg) = self.0.recv().await {
            match msg {
                Msg::Stop => break,
                Msg::Boop => {
                    println!("Boop");
                }
            }
        }
    }
}