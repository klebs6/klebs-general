// named-item-derive/tests/trybuild/fail_generics_not_used.rs

#![allow(unused)]
use named_item_derive::NamedItem;
use named_item::*;

// If we want to show that you can define a generic param
// but not actually store it, you might get some warnings or errors:
#[derive(NamedItem)]
pub struct UnusedGeneric<T> {
    name: String,
    // We do not store `T` anywhere else...
}

fn main() {}

