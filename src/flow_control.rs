use std::{intrinsics::transmute, marker::PhantomData, ops::Deref};

mod levels {
	pub(crate) struct HighSec;
	pub(crate) struct LowSec;
}

// The use of #[repr(transparent)] ensures that the layout of Item is stable across transmutations
// of the marker type.
#[repr(transparent)]
#[derive(Debug, Clone, Default)]
struct Item<T, Level> {
	t:       Box<T>,
	_marker: PhantomData<Level>,
}

impl<T> Item<T, levels::LowSec> {
	pub fn low_sec(t: T) -> Self { Self { t: Box::new(t), _marker: PhantomData } }

	pub fn high_sec(t: T) -> Item<T, levels::HighSec> {
		// naive way, without unsafe: error: mismatched types; label: expected struct `HighSec`, found
		// struct `LowSec`. Need an unsafe block to transmute marker to Highsec.
		unsafe { transmute(Self { t: Box::new(t), _marker: PhantomData }) }
		// other things to try:
		// try just transmute inside? transmute doesn't kno what to do
		// unsafe{Self { t: Box::new(t),_marker: transmute(PhantomData) }}
		// unsafe{Self { t: Box::new(t), transmute(_marker: PhantomData) }} // syntax error
		// Nope, unsafe block has to be outside
		// Self { t: Box::new(t), unsafe{transmute(_marker: PhantomData)}}
	}
}

impl<T, Level> Deref for Item<T, Level> {
	type Target = T;

	fn deref(&self) -> &T { &self.t }
}

// guarantee layout of SecureVec is stable across transmutations of the marker
#[repr(transparent)]
struct SecureVec<T, Level> {
	items:   Vec<Item<T, Level>>,
	_marker: PhantomData<Level>,
}

// impl {new, get,push} for low security vector
// here we implemented 3 different push methods. That kinda sucks, is there a way to get around
// that? See module below, flow_control_alt.
impl<T> SecureVec<T, levels::LowSec> {
	pub fn new() -> Self { Self { items: Vec::new(), _marker: PhantomData } }

	pub fn get(&self, i: usize) -> &T { &self.items[i] }

	pub fn push(&mut self, item: Item<T, levels::LowSec>) { self.items.push(item) }

	pub fn push_secure(self, item: Item<T, levels::HighSec>) -> SecureVec<T, levels::HighSec> {
		// don't do this:
		// unsafe { transmute(self) };
		// self.items.push(item);
		// self
		// do this:
		let mut vec: SecureVec<T, levels::HighSec> = unsafe { transmute(self) };
		vec.items.push(item);
		vec
	}
}

// a "witness" to security, representing a logged in user
struct HighSecWitness;
impl HighSecWitness {
	// sprinkle some high-security authentication in here...
	pub fn login(/* some credentials */) -> HighSecWitness { HighSecWitness }
}

impl<T> SecureVec<T, levels::HighSec> {
	pub fn get_secure(&self, i: usize, _witness: HighSecWitness) -> &T { &self.items[i] }

	pub fn push_secure(&mut self, value: T, _witness: HighSecWitness) {
		self.items.push(Item::high_sec(value))
	}
	// fails: expected high_sec item
	// pub fn push_insecure(&mut self, value: T, _witness: HighSecWitness)  {
	// self.items.push(Item::low_sec(value)) }
}

#[test]
fn test_security() {
	let mut v = SecureVec::new();
	let lo = Item::low_sec(1);
	let hi = Item::high_sec(2);
	v.push(lo); // v is still low sec
	assert_eq!(*v.get(0), 1); // ok to read v

	let v = v.push_secure(hi); // v is now high sec

	// assert_eq!(v.get(0), 1); // can't read any more, compiler error
	let w = HighSecWitness::login();
	assert_eq!(*v.get_secure(1, w), 2); // can read after login
}

mod flow_control_alt {
	use std::{intrinsics::transmute, marker::PhantomData, ops::Deref};

	mod levels {
		pub(crate) struct HighSec;
		pub(crate) struct LowSec;
	}

	#[repr(transparent)]
	#[derive(Debug, Clone, Default)]
	struct Item<T, Level> {
		t:       Box<T>,
		_marker: PhantomData<Level>,
	}

	impl<T> Item<T, levels::LowSec> {
		pub fn low_sec(t: T) -> Self { Self { t: Box::new(t), _marker: PhantomData } }

		pub fn high_sec(t: T) -> Item<T, levels::HighSec> {
			unsafe { transmute(Self { t: Box::new(t), _marker: PhantomData }) }
		}
	}

	impl<T, Level> Deref for Item<T, Level> {
		type Target = T;

		fn deref(&self) -> &T { &self.t }
	}

	#[repr(transparent)]
	struct SecureVec<T, Level> {
		items:   Vec<Item<T, Level>>,
		_marker: PhantomData<Level>,
	}

	// Instead of implementing push a bunch of times, use a type operator to define the Maxmimum
	// security: traits in type-level programming are like functions.
	// the generic inputs are like the function inputs, and the associated types are their outputs.
	mod compute_level {
		use super::levels::*;
		pub trait ComputeMaxLevel<InputLevel> {
			type OutputLevel;
		}
		impl ComputeMaxLevel<LowSec> for LowSec {
			type OutputLevel = LowSec;
		}
		impl ComputeMaxLevel<HighSec> for LowSec {
			type OutputLevel = HighSec;
		}
		impl ComputeMaxLevel<LowSec> for HighSec {
			type OutputLevel = HighSec;
		}
		impl ComputeMaxLevel<HighSec> for HighSec {
			type OutputLevel = HighSec;
		}
		// The type alias gives us a more convenient way to "call" the type operator
		// This says, treat "L" as trait object ComputeMaxLevel<R>, and define MaxLevel<L,R> to be its
		// OutputLevel
		pub type MaxLevel<L, R> = <L as ComputeMaxLevel<R>>::OutputLevel;
	}
	use compute_level::{ComputeMaxLevel, MaxLevel};
	impl<T, VecLevel> SecureVec<T, VecLevel> {
		pub fn push<ItemLevel>(
			mut self,
			item: Item<T, ItemLevel>,
		) -> SecureVec<T, MaxLevel<ItemLevel, VecLevel>>
		where
			ItemLevel: ComputeMaxLevel<VecLevel>,
		{
			unsafe {
				// transmute the item to the correct level
				self.items.push(transmute(item));
				// and return self at the correct level
				transmute(self)
			}
		}
	}

	impl<T> SecureVec<T, levels::LowSec> {
		pub fn new() -> Self { Self { items: Vec::new(), _marker: PhantomData } }

		pub fn get(&self, i: usize) -> &T { &self.items[i] }
	}

	struct HighSecWitness;
	impl HighSecWitness {
		pub fn login(/* some credentials */) -> HighSecWitness { HighSecWitness }
	}

	impl<T> SecureVec<T, levels::HighSec> {
		pub fn get_secure(&self, i: usize, _witness: HighSecWitness) -> &T { &self.items[i] }
	}

	#[test]
	fn test_max_level() {
		let v = SecureVec::new();
		let lo = Item::low_sec(1);
		let hi = Item::high_sec(2);
		let v = v.push(lo); // v is still low sec
		assert_eq!(*v.get(0), 1); // ok to read v

		let v = v.push(hi); // v is now high sec

		// assert_eq!(v.get(0), 1); // can't read any more, compiler error
		let w = HighSecWitness::login();
		assert_eq!(*v.get_secure(1, w), 2); // can read after login
	}
}
