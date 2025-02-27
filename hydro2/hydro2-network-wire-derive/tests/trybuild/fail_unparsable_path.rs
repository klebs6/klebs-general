// ---------------- [ File: hydro2-network-wire-derive/tests/trybuild/fail_unparsable_path.rs ]
// tests/trybuild/fail_unparsable_path.rs

use hydro2_network_wire_derive::NetworkWire;

#[derive(NetworkWire)]
#[available_operators(
   op="SomeOp("  // This is invalid path syntax
)]
pub struct MyWire;

fn main() {}
