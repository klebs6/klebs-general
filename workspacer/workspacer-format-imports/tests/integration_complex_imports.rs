// ---------------- [ File: workspacer-format-imports/tests/integration_complex_imports.rs ]
use workspacer_format_imports::*;
use workspacer_3p::*;

#[traced_test]
fn test_integration_complex_nested_imports() {
    info!("test_integration_complex_nested_imports => start");

    // A big, complex input featuring multiple consecutive comments,
    // blank lines, nested and braced imports, various visibility, etc.
    let input = r#"
// Above line 1
// Above line 2

pub(crate) use crate::foo::prelude::*;
pub use crate::foo::prelude::{Zeta, Beta, Alpha};

// Block comment for next import
// There's a blank line, so the above won't attach

use   crate::nested::{
    mod1::{Thing1, Thing2},
    mod2::{inner::{DeepStuff}},
};

/// Doc comment might appear in code
fn main() {
    println!("Hello, world!");
}

pub(super) use crate::something_else::random;
pub use crate::something_else::{SOME_CONST, AnotherType};

use std::io::Result; // trailing comment

    // Another function
    fn something_unrelated() { /* not relevant to imports */ }
"#;
    debug!("Input:\n{}", input);

    // Execute the formatting
    let result = sort_and_format_imports_in_text(input)
        .expect("Expected successful formatting of complex input");

    debug!("Formatted Result:\n{}", result);

    // Let’s do some key assertions:

    // 1) We should see "pub(crate) use crate::foo::prelude::*;" still present.
    assert!(
        result.contains("pub(crate) use crate::foo::prelude::*;"),
        "Missing the wildcard prelude import"
    );

    // 2) Zeta, Beta, Alpha should remain in a braced group after 'crate::foo::prelude'
    //    in alphabetical order or however your code organizes them.
    //    We'll just check that all appear in a single import line.
    assert!(
        result.contains("use crate::foo::prelude::{Alpha, Beta, Zeta};")
            || result.contains("use crate::foo::prelude::{Zeta, Beta, Alpha};"),
        "Missing or incorrectly grouped Alpha, Beta, Zeta braced import"
    );

    // 3) The mod1 and mod2 items might get grouped: 
    //    "use crate::nested::{mod1::{Thing1, Thing2}, mod2::{inner::{DeepStuff}}};"
    //    We'll just ensure they're still braced together in some fashion.
    assert!(result.contains("use crate::nested::{mod1::{Thing1, Thing2}, mod2::{inner::{DeepStuff}}};")
        || result.contains("use crate::nested::{mod2::{inner::{DeepStuff}}, mod1::{Thing1, Thing2}}};"),
        "Nested mod1/mod2 items not grouped or missing"
    );

    // 4) The doc comment near `fn main()` doesn't vanish.
    //    We'll check that the doc comment is still somewhere in the result.
    assert!(
        result.contains("/// Doc comment might appear in code"),
        "Doc comment near main() is missing from the final output"
    );

    // 5) The import of 'something_else::{SOME_CONST, AnotherType}' 
    //    should remain braced together.
    assert!(
        result.contains("use crate::something_else::{AnotherType, SOME_CONST};")
            || result.contains("use crate::something_else::{SOME_CONST, AnotherType};"),
        "Missing or incorrectly braced AnotherType + SOME_CONST"
    );

    // 6) Check that the trailing comment after 'use std::io::Result'
    //    is preserved near that import line.
    //    We can do a quick check that the final text still has 'use std::io::Result;' 
    //    and the comment on the same line or next line.
    //    For simplicity, just check that the line is present in the output at all.
    assert!(
        result.contains("use std::io::Result; // trailing comment"),
        "Trailing comment after 'use std::io::Result' should remain"
    );

    // 7) The blank line after "Block comment for next import" 
    //    means that comment won't attach to the subsequent 'use crate::nested...'.
    //    We'll confirm that the comment isn't found in the final above the nested import line.
    //    (We won't do a strict textual check, but we'll ensure it didn't glom on.)
    let block_comment_idx = result.find("// There's a blank line, so the above won't attach");
    if let Some(idx) = block_comment_idx {
        // We'll look for the nested import *right after* that index. 
        // If they're separated by a blank line, the code is correct.
        let snippet_after = &result[idx..];
        assert!(!snippet_after.contains("use crate::nested"));
    }

    // 8) We expect the wildcard 'pub (super) use crate::something_else::random;' 
    //    to remain.
    assert!(
        result.contains("pub(super) use crate::something_else::random;"),
        "pub(super) import was missing"
    );

    info!("test_integration_complex_nested_imports => success");
}
