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

pub enum Call<State> {
    Ref(fn(&State)),
    RefMut(fn(&mut State)),
}

pub struct Process<State>
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

type CallReceiver<State> = UnboundedReceiver<Call<State>>;
type CallSender<State> = UnboundedSender<Call<State>>;
type ReplySender<Reply> = oneshot::Sender<Reply>;

impl<State> Process<State>
where
    State: Debug,
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
                Call::Ref(caller) => {
                    caller(&self.state);
                },
                Call::RefMut(caller) => {
                    caller(&mut self.state);
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

    pub fn call_ref(&self, caller: fn(&State)) {
        self.sender.send(Call::Ref(caller)).ok();
    }

    pub fn call_ref_mut(&self, caller: fn(&mut State)) {
        self.sender.send(Call::RefMut(caller)).ok();
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
            toggle.call_ref(|state| {
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
        self.call_ref_mut(|state| {
            println!("state: {:?}", state);
            match state {
               Toggle::Alpha => *state = Toggle::Beta,
               Toggle::Beta => *state = Toggle::Alpha,
            }
        });
    }
}
