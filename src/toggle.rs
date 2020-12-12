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

pub enum Error {
    NotSent,
    NoReply,
}

pub type Result<Reply> = std::result::Result<Reply, Error>;

pub enum Call<State, Reply> {
    Ref(fn(&State)),
    RefMut(fn(&mut State)),
    RefReply(fn(&State, ReplySender<Reply>), ReplySender<Reply>),
    RefMutReply(fn(&State, ReplySender<Reply>), ReplySender<Reply>),
}

pub struct Process<State, Reply>
{
    state: State,
    receiver: CallReceiver<State, Reply>,
}

pub struct Actor<State, Reply> {
    sender: CallSender<State, Reply>
}

impl<State, Reply> Clone for Actor<State, Reply> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

type CallReceiver<State, Reply> = UnboundedReceiver<Call<State, Reply>>;
type CallSender<State, Reply> = UnboundedSender<Call<State, Reply>>;
type ReplyReceiver<Reply> = oneshot::Receiver<Reply>;
type ReplySender<Reply> = oneshot::Sender<Reply>;

impl<State, Reply> Process<State, Reply>
where
    State: Debug,
{
    pub fn new_with_state(state: State) -> (Self, Actor<State, Reply>) {
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
        while let Some(call) = self.receiver.recv().await {
            match call {
                Call::Ref(caller) => {
                    caller(&self.state);
                },
                Call::RefMut(caller) => {
                    caller(&mut self.state);
                },
                Call::RefReply(caller, reply_sender) => {
                    caller(&self.state, reply_sender);
                },
                Call::RefMutReply(caller, reply_sender) => {
                    caller(&mut self.state, reply_sender);
                }
            }
        }
    }
}

impl<State, Reply> Actor<State, Reply>
where
    State: Debug,
{
    pub fn new_with_sender(sender: CallSender<State, Reply>) -> Self {
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

    pub async fn call_ref_reply(&self, caller: fn(&State, ReplySender<Reply>)) -> Reply {
        let (reply_sender, reply_receiver) = oneshot::channel();
        self.sender.send(Call::RefReply(caller, reply_sender)).ok();

        reply_receiver.await.ok().unwrap()
    }

    pub async fn call_ref_mut_reply(&self, caller: fn(&State, ReplySender<Reply>)) -> Reply {
        let (reply_sender, reply_receiver) = oneshot::channel();
        self.sender.send(Call::RefMutReply(caller, reply_sender)).ok();

        reply_receiver.await.ok().unwrap()
    }
}

//
// USAGE
//
pub async fn exercise_toggle() {
    let (mut process, toggle) = Process::<Toggle, Toggle>::new_with_state(Toggle::Alpha);

    let toggle_clone = toggle.clone();

    let (process, task1, task2) = tokio::join! {
        async move {
            process.start().await;
        },
        async move {
            println!("received reply: {:?}", toggle.call_ref_reply(|state, reply| {
                println!("sending reply: {:?}", state);
                reply.send(*state).ok();
            }).await);

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

#[derive(Debug, Copy, Clone)]
enum Toggle {
    Alpha,
    Beta,
}

impl Actor<Toggle, Toggle> {
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
