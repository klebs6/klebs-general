// ---------------- [ File: workspacer-format-imports/tests/integration_format_imports.rs ]
use workspacer_format_imports::sort_and_format_imports_in_text;
use workspacer_3p::*;

#[traced_test]
fn test_integration_single_use_no_comment() {
    info!("test_integration_single_use_no_comment => start");
    let input = r#"
fn main() {
    // just some code
}

use std::collections::HashMap;

fn foo() { }
"#;

    debug!("Input:\n{}", input);

    // The only `use` statement is `use std::collections::HashMap;`
    // We'll run it through the sort_and_format_imports_in_text function
    let result = sort_and_format_imports_in_text(input)
        .expect("Expected successful formatting");

    debug!("Formatted Result:\n{}", result);

    // Since there's only one import, we should see it remain basically the same (plus possibly a newline).
    // We'll ensure it doesn't break anything.
    assert!(result.contains("use std::collections::HashMap;"));
    info!("test_integration_single_use_no_comment => success");
}

#[traced_test]
fn test_integration_multiple_uses_with_comments() {
    info!("test_integration_multiple_uses_with_comments => start");
    let input = r#"
// The above file comment

use std::io::Result;
use std::io::Error;

// Another above line
pub use crate::something::Foo;

fn main() {
    // example
}
"#;
    debug!("Input:\n{}", input);

    // We have two imports:
    //   1) use std::io::Result;
    //   2) use std::io::Error;
    // plus a "pub use crate::something::Foo;" with a preceding comment.
    // We'll see them get grouped by prefix => "std::io::{Error, Result}" is typical.
    let result = sort_and_format_imports_in_text(input)
        .expect("Expected successful formatting");

    debug!("Formatted Result:\n{}", result);

    // We expect something like:
    //   // The above file comment
    //   // Another above line
    //   pub use crate::something::Foo;
    //   use std::io::{Error, Result};
    // Or the reverse order, depending on your sorting logic (alphabetical).
    // We'll just check that both lines exist and the grouping is correct.

    assert!(
        result.contains("pub use crate::something::Foo;"),
        "Missing the 'pub use' line"
    );
    // Could be "use std::io::{Error, Result};" (alphabetical means Error before Result or vice versa).
    assert!(
        result.contains("use std::io::{Error, Result};")
        || result.contains("use std::io::{Result, Error};"),
        "Missing the grouped std::io line"
    );

    // Ensure the file-level comment lines remain
    assert!(result.contains("// The above file comment"));
    assert!(result.contains("// Another above line"));

    info!("test_integration_multiple_uses_with_comments => success");
}

#[traced_test]
fn test_integration_use_with_multiple_consecutive_comments() {
    info!("test_integration_use_with_multiple_consecutive_comments => start");
    let input = r#"
// First line comment
// Second line comment
use std::fs::File;

fn main(){}
"#;
    debug!("Input:\n{}", input);

    let result = sort_and_format_imports_in_text(input)
        .expect("Expected successful formatting");

    debug!("Formatted Result:\n{}", result);

    // We expect to preserve both consecutive comment lines directly above the `use`.
    assert!(result.contains("// First line comment"));
    assert!(result.contains("// Second line comment"));
    assert!(result.contains("use std::fs::File;"));

    info!("test_integration_use_with_multiple_consecutive_comments => success");
}

#[traced_test]
fn test_integration_blank_line_blocks_comment() {
    info!("test_integration_blank_line_blocks_comment => start");
    let input = r#"
// This is blocked by blank line

use crate::blocked::Thing;
"#;
    debug!("Input:\n{}", input);

    let result = sort_and_format_imports_in_text(input)
        .expect("Expected successful formatting");

    debug!("Formatted Result:\n{}", result);

    // Because there's a blank line after "// This is blocked by blank line", 
    // that comment does *not* attach to the `use`.
    // So we do *not* expect that line to appear above the final 'use crate::blocked::Thing;' 
    // in the sorted block. We'll just confirm it doesn't appear with it.

    // The comment is presumably lost (or rather, not considered "leading" for the use).
    // We'll ensure that the sorted imports don't contain that line as a leading comment.
    assert!(result.contains("use crate::blocked::Thing;"));
    let idx_import = result.find("use crate::blocked::Thing;").unwrap();
    let snippet_before = &result[..idx_import];
    assert!(
        !snippet_before.contains("blocked by blank line"),
        "Should NOT attach that comment to the use"
    );

    info!("test_integration_blank_line_blocks_comment => success");
}
