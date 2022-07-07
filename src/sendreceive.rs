// 1. For each acceptable state, make a unique unit struct. For clarity, I'm going to hold them in
// their own mod.
mod states {
	pub(crate) struct Receiving;
	pub(crate) struct Sending;
}
use std::{
	intrinsics::transmute,
	marker::PhantomData,
	sync::mpsc::{self, channel, Receiver, Sender},
};

use states::*;

// 2. Make a state-holder struct, parameterized my state structs
// The state machine is parameterized by the state
#[repr(transparent)]
struct Channel<State, T> {
	chan:   T,
	_state: PhantomData<State>,
}

// Methods for the state are uniquely associated with only the state
impl Channel<Receiving, Receiver<String>> {
	// recv consumes ownership, ensuring old state is invalidated
	fn recv(self) -> (Channel<Sending, Sender<String>>, String) {
		let msg = self.chan.recv().unwrap();
		// The state type changes after executing a transition
		(unsafe { transmute(self) }, msg)
	}
}

impl Channel<Sending, Sender<String>> {
	fn new() -> Self {
		let (tx, _rx) = channel();
		Self { chan: tx, _state: PhantomData }
	}

	fn send(self, msg: String) -> Channel<Receiving, Receiver<String>> {
		self.chan.send(msg).unwrap();
		unsafe { transmute(self) }
	}
}

#[test]
fn channel_test() {
	let c: Channel<Sending, Sender<String>> = Channel::new();
	let c = c.send("hi".into());
	let (c, msg) = c.recv();
	// let (c, msg) = c.recv(); // error! must send.
	// and so on
}
