// ---------------- [ File: tests/ui/fail_missing_name_field.rs ]
// named-item-derive/tests/trybuild/fail_missing_name_field.rs

#![allow(unused)]
use named_item_derive::NamedItem;
use named_item::*; // Named, etc.

#[derive(NamedItem)]
pub struct BadStruct {
    // We do not define `name: String`
    // => should fail with "Struct must have `name: String`."
    foo: i32,
}

fn main() {}
