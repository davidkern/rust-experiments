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
    Inspect(fn(&State)),
    Mutate(fn(&mut State)),
}

pub trait Inspectable {
    fn inspect(&self, inspector: fn(&Self)) {
        inspector(self);
    }
}

pub trait Mutable
{
    fn mutate(&mut self, mutator: fn(&mut Self)) {
        mutator(self);
    }
}

pub struct Process<State>
where
    State: Mutable,
{
    state: State,
    receiver: Receiver<State>,
}

pub struct Actor<State> {
    sender: Sender<State>
}

impl<State> Clone for Actor<State> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

type Receiver<State> = UnboundedReceiver<Op<State>>;
type Sender<State> = UnboundedSender<Op<State>>;

impl<State> Process<State>
where
    State: Inspectable + Mutable + Debug,
{
    pub fn new_with_state(state: State) -> (Self, Actor<State>) {
        let (sender, receiver) = unbounded_channel();
        (
            Self {
                state,
                receiver,
            },
            Actor::new_with_sender(sender)
        )
    }

    pub async fn start(&mut self) {
        while let Some(op) = self.receiver.recv().await {
            match op {
                Op::Inspect(inspector) => {
                    self.state.inspect(inspector);
                },
                Op::Mutate(mutator) => {
                    self.state.mutate(mutator);
                }
            }
        }
    }
}

impl<State> Actor<State>
where
    State: Debug,
{
    pub fn new_with_sender(sender: Sender<State>) -> Self {
        Self {
            sender,
        }
    }

    pub fn inspect(&self, inspector: fn(&State)) {
        self.sender.send(Op::Inspect(inspector)).ok();
    }

    pub fn mutate(&self, mutator: fn(&mut State)) {
        self.sender.send(Op::Mutate(mutator)).ok();
    }

    // pub async fn mutate_and_reply(&self, mutator: fn(&mut State)) -> &State {
    //
    // }
}

//
// USAGE
//
pub async fn exercise_toggle() {
    let (mut process, toggle) = Process::<Toggle>::new_with_state(Toggle::Alpha);

    let toggle_clone = toggle.clone();

    let (state, p1, p2) = tokio::join! {
        async move {
            process.start().await;
        },
        async move {
            toggle.inspect(|state| {
                println!("inspect: {:?}", state);
            });

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

impl Inspectable for Toggle { }
impl Mutable for Toggle { }

impl Actor<Toggle> {
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
