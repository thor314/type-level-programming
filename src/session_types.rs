// Let's implement session types, because I had nothing else to do today

use std::marker::PhantomData;

// A grammar:
/// Send message type T, first choice in Offer. Send either a Ping or a Receive Ping|Goto<Z>
struct Send<T, S>(PhantomData<(T, S)>);
/// Receive message type T
struct Recv<T, S>(PhantomData<(T, S)>);
/// Offer sub-protocol, a pair of subroutine choices. One is probably end.
struct Offer<Left, Right>(PhantomData<(Left, Right)>);
/// Choose a sub-protocol, the Dual of Offer? More below.
struct Choose<Left, Right>(PhantomData<(Left, Right)>);
/// Label point for Goto to jump to
struct Label<S>(PhantomData<S>);
/// Go to some Label
struct Goto<N>(PhantomData<N>);
/// An encoding of Integers
struct Z;
struct S<N>(PhantomData<N>); // Peano encoding for natural numbers
/// Second choice in Offer, signal to close the chanel
struct Close;

struct Ping;
type PingServer =
  Label<
    Offer<
      Send<Ping,
        Recv<Ping,
        Goto<Z>>>,
      Close>>;

// Without an implementation of `Chan`, this gets very hard to follow. Leave here.
// fn example_ping_server() {
//   let (c, _): (Chan<(), PingServer>,
//                Chan<(), Dual<PingServer>) = Chan::new();
//   let mut c: Chan<(Offer<_,_>, ()), Offer<_,_>> = c.label();
//   loop {
//     c = match c.offer() {
//       Branch::Left(c) => {
//         let c: Chan<_, Recv<_,_>> = c.send(Ping);
//         let (c, Ping): (Chan<_, Goto<_>>, _) = c.recv();
//         c.goto()
//       },
//       Branch::Right(c) => {
//         c.close();
//         return;
//       }
//     }
//   }
// }
