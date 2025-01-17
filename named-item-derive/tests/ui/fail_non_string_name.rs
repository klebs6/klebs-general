// named-item-derive/tests/trybuild/fail_non_string_name.rs

#![allow(unused)]
use named_item_derive::NamedItem;
use named_item::*;

#[derive(NamedItem)]
pub struct WrongNameField {
    name: i32,  // not `String`
}

fn main() {}

