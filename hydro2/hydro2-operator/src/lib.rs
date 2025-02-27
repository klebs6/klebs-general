// ---------------- [ File: hydro2-operator/src/lib.rs ]
#![feature(trait_alias)]
#[macro_use] mod imports; use imports::*;

x!{errors}
x!{opcode}
x!{operator}
x!{port_try_into}
x!{port_try_from}
x!{port_try_into_any}
//x!{operator_i_o} // this doesn't quite work yet
pub use unsafe_erased;
pub use async_trait;
