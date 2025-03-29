// ---------------- [ File: tests/ui/fail_missing_history.rs ]
// Should fail because `history="true"` but no name_history field is present

use named_item_derive::NamedItem;

#[derive(NamedItem)]
#[named_item(history="true")]
struct MissingHistoryField {
    name: String,
}

fn main() {}
