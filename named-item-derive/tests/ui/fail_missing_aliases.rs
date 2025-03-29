// ---------------- [ File: tests/ui/fail_missing_aliases.rs ]
// Should fail because `aliases="true"` but no aliases: Vec<String> field

use named_item_derive::NamedItem;

#[derive(NamedItem)]
#[named_item(aliases="true")]
struct MissingAliasesField {
    name: String,
}

fn main() {}
