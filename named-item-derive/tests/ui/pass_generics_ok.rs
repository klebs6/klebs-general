// named-item-derive/tests/trybuild/pass_generics_ok.rs

#![allow(unused)]
use named_item_derive::NamedItem;
use named_item::*;

#[derive(NamedItem)]
pub struct GenericEntity<T> {
    // required field:
    name: String,
    // if we set `aliases="true"` in named_item, we'd need `aliases: Vec<String>`
    // if we set `history="true"`, we'd need `name_history: Vec<String>`

    value: T, 
}

fn main() {
    // If it compiles => success
    let mut x = GenericEntity::<i32> {
        name: "foo".to_string(),
        value: 42,
    };
    // We can call Named trait:
    assert_eq!(x.name(), "foo");
    // We can call set_name if we like: (since we didn't set `history="true"`, we still have a SetName impl)
    x.set_name("bar").unwrap();
    assert_eq!(x.name(), "bar");
}

