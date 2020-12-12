//struct Actor<T> {}

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use std::ops::Deref;
use crate::System;

//
// IMPLEMENTATION
//

pub struct Actor<Message, State> {
    state: State,
    receiver: Receiver<Message>,
}

pub struct Mailbox<Message> {
    actor: Sender<Message>
}

type Receiver<State> = UnboundedReceiver<State>;
type Sender<State> = UnboundedSender<State>;

impl<Message, State> Actor<Message, State> {
    pub fn new_with_state(state: State) -> (Self, Mailbox<Message>) {
        let (sender, receiver) = unbounded_channel();
        let actor = Self {
            state,
            receiver,
        };
        let mailbox = Mailbox::new_with_sender(sender);

        (actor, mailbox)
    }
}

impl<Message> Mailbox<Message> {
    pub fn new_with_sender(sender: Sender<Message>) -> Self {
        Self {
            actor: sender,
        }
    }
}

//
// USAGE
//

fn exercise_toggle() {

}

struct Toggle;

enum ToggleState {
    Alpha,
    Beta,
}

impl Actor<Toggle, ToggleState> {
    async fn toggle(&self) {
    }
}

// // INFINITE LOOP with types
// // Type definition toggles as the state toggles (!)
// pub type Receiver = UnboundedReceiver<State>;
// pub struct Toggle(UnboundedSender<State>);
//
// struct ToggleActor(Receiver);
//
// enum State {
//     Alpha,
//     Beta,
// }
//
// impl Deref for Toggle {
//     type Target = UnboundedSender<State>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl Toggle {
//     pub fn actor() -> (Toggle, State) {
//         let (sender, receiver) = unbounded_channel();
//
//
//         (Toggle(sender), State::Alpha)
//     }
//
//     pub async fn start(&mut self) {
//         while let Some(msg) = self.recv().await {
//             self.state()
//         }
//     }
//
//     pub fn state(self) -> Self {
//         match self {
//             Self::Alpha => Self::Beta,
//             Self::Beta => Self::Alpha,
//         }
//     }
//
//     pub fn toggle(&self) -> &Self {
//
//     }
// }
