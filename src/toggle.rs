//struct Actor<T> {}

use tokio::sync::mpsc::{
    UnboundedSender,
    unbounded_channel,
    UnboundedReceiver
};
use tokio::sync::oneshot;
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

impl<State> Inspectable for State { }
impl<State> Mutable for State { }
//impl<State, Data> Measurable<Data> for State { }

pub struct Process<State>
where
    State: Mutable,
{
    state: State,
    receiver: CallReceiver<State>,
}

pub struct Actor<State> {
    sender: CallSender<State>
}

impl<State> Clone for Actor<State> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

type CallReceiver<State> = UnboundedReceiver<Op<State>>;
type CallSender<State> = UnboundedSender<Op<State>>;
type ReplySender<Reply> = oneshot::Sender<Reply>;

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
                },
            }
        }
    }
}

impl<State> Actor<State>
where
    State: Debug,
{
    pub fn new_with_sender(sender: CallSender<State>) -> Self {
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

    let (process, task1, task2) = tokio::join! {
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
