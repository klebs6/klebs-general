// ---------------- [ File: tests/trybuild/fail_no_op_keyword.rs ]
// tests/trybuild/fail_no_op_keyword.rs

use hydro2_network_wire_derive::NetworkWire;

#[derive(NetworkWire)]
#[available_operators(foo="Bar")]
pub struct MyWire;

fn main() {}
