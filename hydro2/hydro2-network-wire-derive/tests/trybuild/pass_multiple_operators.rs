// ---------------- [ File: hydro2-network-wire-derive/tests/trybuild/pass_multiple_operators.rs ]
// tests/trybuild/pass_multiple_operators.rs

use hydro2_network_wire_derive::*;
use hydro2_basic_operators::*;
use hydro2_operator::*;
use std::marker::PhantomData;
use std::ops::*;
use std::fmt::{Debug,Display};

#[derive(Debug,Clone,NetworkWire)]
#[available_operators(
    op="ConstantOp<T>",
    op="DoubleOutOp",
    op="DoubleToTriOp<U>",
    op="DoubleToTriTwoGenericsOp<T,U>"
)]
pub struct BigWireA<T,U> 

where U: PartialEq + Copy + Display + Mul<U,Output=U> + Add<U,Output=U> + Sub<U,Output=U> + Debug + Send + Sync,
      T: PartialEq + Copy + Display + Mul<T,Output=T> + Add<T,Output=T> + Sub<T,Output=T> + Debug + Send + Sync,
{
    _t: PhantomData<T>,
    _u: PhantomData<U>,
}

fn main() {}
