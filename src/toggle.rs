//struct Actor<T> {}

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};
use std::ops::Deref;
use crate::{System, State};
use std::future::Future;
use std::fmt::Debug;

//
// IMPLEMENTATION
//

pub enum Op<State> {
    Mutate(fn(&mut State))
}

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

impl<State> Clone for Mailbox<State> {
    fn clone(&self) -> Self {
        Self {
            actor: self.actor.clone(),
        }
    }
}

type Receiver<State> = UnboundedReceiver<Op<State>>;
type Sender<State> = UnboundedSender<Op<State>>;

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
        while let Some(op) = self.receiver.recv().await {
            match op {
                Op::Mutate(mutator) => {
                    self.state.mutate(mutator);
                }
            }
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
        self.actor.send(Op::Mutate(mutator)).ok();
    }

    // pub async fn mutate_and_reply(&self, mutator: fn(&mut State)) -> &State {
    //
    // }
}

//
// USAGE
//
pub async fn exercise_toggle() {
    let (mut actor_toggle, toggle) = Actor::<Toggle>::new_with_state(Toggle::Alpha);

    let toggle_clone = toggle.clone();

    let (state, p1, p2) = tokio::join! {
        async move {
            actor_toggle.start().await;
        },
        async move {
            toggle.toggle();
            toggle.toggle();
            toggle.toggle();
            toggle.toggle();
        },
        async move {
            toggle_clone.toggle();
            toggle_clone.toggle();
            toggle_clone.toggle();
            toggle_clone.toggle();
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
