//struct Actor<T> {}

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use std::ops::Deref;
use crate::System;

// INFINITE LOOP with types
// Type definition toggles as the state toggles (!)
pub type Receiver = State<UnboundedReceiver<State, _>>;
pub struct Toggle(UnboundedSender<State<UnboundedReceiver<...>>>);

enum State<Receiver> {
    Alpha(Receiver),
    Beta(Receiver),
}

impl Deref for Toggle {
    type Target = UnboundedSender<State>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Toggle {
    pub fn actor() -> (Toggle, State) {
        let (sender, receiver) = unbounded_channel();


        (Toggle(sender), State::Alpha)
    }

    pub async fn start(&mut self) {
        while let Some(msg) = self.recv().await {
            self.state()
        }
    }

    pub fn state(self) -> Self {
        match self {
            Self::Alpha => Self::Beta,
            Self::Beta => Self::Alpha,
        }
    }

    pub fn toggle(&self) -> &Self {

    }
}
