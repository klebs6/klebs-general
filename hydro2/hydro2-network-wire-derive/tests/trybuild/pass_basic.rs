// ---------------- [ File: hydro2-network-wire-derive/tests/trybuild/pass_basic.rs ]
// tests/trybuild/pass_basic.rs

use hydro2_network_wire_derive::NetworkWire;
use hydro2_basic_operators::*;
use hydro2_operator::*;
use std::marker::PhantomData;
use std::fmt::{self,Display,Debug};

// 2) Also ensure your crate sees `#[available_operators(...)]` as a recognized attribute.
//   - Typically you just need to do `use hydro2_network_wire_derive::*;` or similar if you re-export
//     the attribute. However, if the attribute is only recognized by the `#[derive(NetworkWire)]`
//     internally, that’s fine. Just ensure no scoping conflicts.

#[derive(Debug,Clone,NetworkWire)]
#[available_operators(
    // The macro expansions create references to FooIO, BarIO<Z>,
    // so we must define those types or stubs.
    op="AddOp",
    op="ConstantOp<Z>"
)]
pub struct MyWireA<Z:ExampleTrait> 
where Z: Display + Copy + Debug + Send + Sync + PartialEq,
{
    _p: PhantomData<Z>,
}

pub trait ExampleTrait {}

#[derive(PartialEq,Eq,Clone,Copy,Debug,Default)]
pub struct ExampleZ;

impl Display for ExampleZ {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f,"ExampleZ")
    }
}


impl ExampleTrait for ExampleZ {}

fn main() {
    // If everything is correct, we compile without error.
    // The macro expansions produce an enum MyWireAIO<Z> with variants:
    //   FooIO(FooIO), BarIO(BarIO<Z>)
    // referencing the stubs we defined above.
    let _ = MyWireAIO::<ExampleZ>::AddOpIO(AddOpIO::default());
    let _ = MyWireAIO::<ExampleZ>::ConstantOpIO(ConstantOpIO::<ExampleZ>::default());
}
