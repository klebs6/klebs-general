
// Should fail because there's no `name: String` field
// We expect the macro to error out.

use named_item_derive::NamedItem;
use named_item::{Named};

#[derive(NamedItem)]
struct MissingNameField {
    something_else: i32,
}

fn main() {}

