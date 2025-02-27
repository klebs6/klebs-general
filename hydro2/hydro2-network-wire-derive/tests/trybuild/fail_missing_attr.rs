// ---------------- [ File: hydro2-network-wire-derive/tests/trybuild/fail_missing_attr.rs ]
// tests/trybuild/fail_missing_attr.rs

use hydro2_network_wire_derive::NetworkWire;
use std::marker::PhantomData;

#[derive(NetworkWire)]
// No `#[available_operators(...)]`, so we expect a compile error.
pub struct MissingOperators<T> {
    _p: PhantomData<T>,
}

fn main() {}

