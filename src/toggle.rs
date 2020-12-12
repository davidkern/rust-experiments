//struct Actor<T> {}

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use std::ops::Deref;
use crate::System;
use std::future::Future;

//
// IMPLEMENTATION
//

pub trait ActorState<Message> {
    fn receive(&mut self, msg: Message) {
    }
}

pub struct Actor<State, Message> {
    state: State,
    receiver: Receiver<Message>,
}

pub struct Mailbox<Message> {
    actor: Sender<Message>
}

type Receiver<State> = UnboundedReceiver<State>;
type Sender<State> = UnboundedSender<State>;

impl<State, Message> Actor<State, Message>
where
    State: ActorState<Message>,
{
    pub fn new_with_state(state: State) -> (Self, Mailbox<Message>) {
        let (sender, receiver) = unbounded_channel();
        let actor = Self {
            state,
            receiver,
        };
        let mailbox = Mailbox::new_with_sender(sender);

        (actor, mailbox)
    }

    pub async fn start(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.state.receive(msg);
        }
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
    let (actor_toggle, toggle) = Actor::<ToggleState, Toggle>::new_with_state(ToggleState::Alpha);

    // tokio::spawn(async { actor_toggle.start().await; }).await.unwrap();
    //
    // toggle.toggle().await;
    // toggle.toggle().await;
}

struct Toggle;

enum ToggleState {
    Alpha,
    Beta,
}

impl ActorState<Toggle> for ToggleState { }

impl Actor<Toggle, ToggleState> {
    pub async fn toggle(&self) {
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
