// ---------------- [ File: hydro2-network-wire-derive/tests/trybuild/fail_hybrid_bad_syntax.rs ]
// tests/trybuild/fail_hybrid_bad_syntax.rs

use hydro2_network_wire_derive::NetworkWire;
use std::marker::PhantomData;

#[derive(NetworkWire)]
#[available_operators(
    op="BadOp("
)]
pub struct MyNetworkWireD {
    _p: PhantomData<i32>,
}

fn main() {}

