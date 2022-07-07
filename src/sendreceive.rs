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
struct Channel<State> {
	chan:   (Sender<String>, Receiver<String>),
	_state: PhantomData<State>,
}

// Methods for the state are uniquely associated with only the state
impl Channel<Receiving> {
	// recv consumes ownership, ensuring old state is invalidated
	fn recv(self) -> (Channel<Sending>, String) {
		let msg = self.chan.1.recv().unwrap();
		// The state type changes after executing a transition
		(unsafe { transmute(self) }, msg)
	}
}

impl Channel<Sending> {
	fn new() -> Self {
		let (tx, rx) = channel();
		Self { chan: (tx,rx), _state: PhantomData }
	}

	fn send(self, msg: String) -> Channel<Receiving> {
		self.chan.0.send(msg).unwrap();
		unsafe { transmute(self) }
	}
}

#[test]
fn channel_test() {
	let c: Channel<Sending> = Channel::new();
	let c = c.send("hi".into());
	let (c, msg) = c.recv();
	// let (c, msg) = c.recv(); // error! must send.
	// and so on
}
