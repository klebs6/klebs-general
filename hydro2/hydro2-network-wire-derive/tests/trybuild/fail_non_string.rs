// ---------------- [ File: tests/trybuild/fail_non_string.rs ]
// tests/trybuild/fail_non_string.rs

use hydro2_network_wire_derive::NetworkWire;
use std::marker::PhantomData;

#[derive(NetworkWire)]
#[available_operators(op=123)]
pub struct MyWire<T> {
    _p: PhantomData<T>,
}

fn main() {}
