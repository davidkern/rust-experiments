//struct Actor<T> {}

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use std::ops::Deref;
use crate::{System, State};
use std::future::Future;
use std::fmt::Debug;

//
// IMPLEMENTATION
//

pub trait Mutable
{
    fn mutate(&mut self, mutator: fn(&mut Self)) {
        mutator(self);
    }
}

pub struct Actor<State>
where
    State: Mutable,
{
    state: State,
    receiver: Receiver<State>,
}

pub struct Mailbox<State> {
    actor: Sender<State>
}

type Receiver<State> = UnboundedReceiver<fn(&mut State)>;
type Sender<State> = UnboundedSender<fn(&mut State)>;

impl<State> Actor<State>
where
    State: Mutable + Debug,
{
    pub fn new_with_state(state: State) -> (Self, Mailbox<State>) {
        let (sender, receiver) = unbounded_channel();
        let actor = Self {
            state,
            receiver,
        };
        let mailbox = Mailbox::new_with_sender(sender);

        (actor, mailbox)
    }

    pub async fn start(&mut self) {
        while let Some(mutator) = self.receiver.recv().await {
            self.state.mutate(mutator);
        }
    }
}

impl<State> Mailbox<State>
where
    State: Debug,
{
    pub fn new_with_sender(sender: Sender<State>) -> Self {
        Self {
            actor: sender,
        }
    }

    pub fn mutate(&self, mutator: fn(&mut State)) {
        self.actor.send(mutator).ok();
    }
}

//
// USAGE
//
pub async fn exercise_toggle() {
    let (mut actor_toggle, toggle) = Actor::<Toggle>::new_with_state(Toggle::Alpha);

    let (state, execution) = tokio::join! {
        async move {
            actor_toggle.start().await;
        },
        async move {
            toggle.toggle();
            toggle.toggle();
            toggle.toggle();
            toggle.toggle();
        },
    };
}

#[derive(Debug)]
enum Toggle {
    Alpha,
    Beta,
}

impl Mutable for Toggle { }

impl Mailbox<Toggle> {
    pub fn toggle(&self) {
        self.mutate(|state| {
            println!("state: {:?}", state);
            match state {
               Toggle::Alpha => *state = Toggle::Beta,
               Toggle::Beta => *state = Toggle::Alpha,
            }
        });
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
