// ---------------- [ File: src/module_interface.rs ]
crate::ix!();

// ---------------------------------------------------------------------------
// Representation of a mod block
// ---------------------------------------------------------------------------
#[derive(Getters,Debug)]
#[getset(get="pub")]
pub struct ModuleInterface {
    docs:    Option<String>,
    attrs:   Option<String>,
    mod_name: String,
    items:   Vec<ConsolidatedItem>,
}

impl ModuleInterface {
    pub fn new(docs: Option<String>, attrs: Option<String>, mod_name: String) -> Self {
        Self { docs, attrs, mod_name, items: vec![] }
    }
    pub fn add_item(&mut self, item: ConsolidatedItem) {
        self.items.push(item);
    }
}

impl fmt::Display for ModuleInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.items.is_empty() {
            return Ok(());
        }
        if let Some(attrs) = &self.attrs {
            for line in attrs.lines() {
                writeln!(f, "{}", line)?;
            }
        }
        if let Some(doc_text) = &self.docs {
            writeln!(f, "{}", doc_text)?;
        }
        writeln!(f, "mod {} {{", self.mod_name)?;
        for (i, item) in self.items.iter().enumerate() {
            let display_str = format!("{}", item);
            for line in display_str.lines() {
                writeln!(f, "    {}", line)?;
            }
            if i + 1 < self.items.len() {
                writeln!(f)?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod test_module_interface {
    use super::*;
    use std::fmt;

    // ------------------------------------------------------------------------
    // Test cases for `ModuleInterface` and its `fmt::Display` implementation
    // ------------------------------------------------------------------------

    /// 1) If `items` is empty, `fmt::Display` should produce an empty string (no output).
    #[test]
    fn test_display_no_items_produces_empty_output() {
        let module = ModuleInterface::new(None, None, "empty_mod".to_string());
        let output = format!("{}", module);
        assert!(
            output.is_empty(),
            "Expected empty output when `items` is empty"
        );
    }

    /// 2) If we have items but no docs or attrs, we expect:
    ///   mod <mod_name> {
    ///       ... items ...
    ///   }
    #[test]
    fn test_display_with_items_no_docs_no_attrs() {
        let mut module = ModuleInterface::new(None, None, "my_mod".to_string());
        module.add_item(ConsolidatedItem::MockTest("fn example() {}".to_string()));
        let output = format!("{}", module);

        let expected = r#"mod my_mod {
    fn example() {}
}
"#;
        assert_eq!(output, expected, "Output should wrap the item in mod block");
    }

    /// 3) If we have `docs` and `attrs`, each line is printed before the `mod` line.
    #[test]
    fn test_display_with_docs_and_attrs() {
        let docs = Some("/// This is my module\n/// Another doc line".to_string());
        let attrs = Some("#[allow(dead_code)]\n#[cfg(feature = \"test\")]".to_string());
        let mut module = ModuleInterface::new(docs.clone(), attrs.clone(), "my_mod".to_string());
        module.add_item(ConsolidatedItem::MockTest("struct MyStruct;".to_string()));

        let output = format!("{}", module);

        // We expect lines for attrs, then lines for docs, then `mod my_mod {`, then item, then "}".
        let expected = r#"#[allow(dead_code)]
#[cfg(feature = "test")]
/// This is my module
/// Another doc line
mod my_mod {
    struct MyStruct;
}
"#;
        assert_eq!(
            output, expected,
            "Docs & attrs should appear before mod declaration, each on its own line"
        );
    }

    /// 4) If we have multiple items, each item is followed by a blank line, except the last one.
    #[test]
    fn test_display_with_multiple_items_spacing() {
        let mut module = ModuleInterface::new(None, None, "multi_mod".to_string());
        module.add_item(ConsolidatedItem::MockTest("// Item 1".to_string()));
        module.add_item(ConsolidatedItem::MockTest("// Item 2".to_string()));
        module.add_item(ConsolidatedItem::MockTest("// Item 3".to_string()));

        let output = format!("{}", module);

        // Notice there's a blank line after each item except the last?
        // Actually, from the posted code, there's a blank line after each item except after the last one.
        // The code does "if i+1 < self.items.len() { writeln!(f)?; }"
        let expected = r#"mod multi_mod {
    // Item 1

    // Item 2

    // Item 3
}
"#;
        assert_eq!(output, expected);
    }

    /// 5) Test that line indentation is correct (4 spaces total: "    ") for item lines,
    ///    and no extra blank lines if there's only one item.
    #[test]
    fn test_indentation_and_single_item_line_spacing() {
        let mut module = ModuleInterface::new(None, None, "indented_mod".to_string());
        module.add_item(ConsolidatedItem::MockTest("fn test_fn() {\nprintln!(\"Hello\");\n}".to_string()));

        let output = format!("{}", module);
        let expected = r#"mod indented_mod {
    fn test_fn() {
    println!("Hello");
}
}
"#;
        assert_eq!(
            output, expected,
            "Each line of the item should be indented by 4 spaces"
        );
    }

    /// 6) If doc or attr strings have multiple lines, each line is printed as-is before the mod.
    #[test]
    fn test_multi_line_docs_and_attrs_verbatim() {
        let docs = Some("//! First doc line\n//! Second doc line".to_string());
        let attrs = Some("#![allow(unused)]\n#![no_std]".to_string());
        let mut module = ModuleInterface::new(docs.clone(), attrs.clone(), "verbatim_mod".to_string());
        module.add_item(ConsolidatedItem::MockTest("fn foo() {}".to_string()));

        let output = format!("{}", module);
        let expected = r#"#![allow(unused)]
#![no_std]
//! First doc line
//! Second doc line
mod verbatim_mod {
    fn foo() {}
}
"#;
        assert_eq!(
            output, expected,
            "Should preserve line-by-line printing of docs and attrs"
        );
    }

    /// 7) If the only doc or attr lines are whitespace or empty, we still print them as-is.
    #[test]
    fn test_doc_attr_whitespace_still_printed() {
        let docs = Some("   ".to_string()); // just spaces
        let attrs = Some("".to_string());   // empty line
        let mut module = ModuleInterface::new(docs.clone(), attrs.clone(), "white_mod".to_string());
        module.add_item(ConsolidatedItem::MockTest("fn white() {}".to_string()));

        let output = format!("{}", module);
        let expected = r#"    

mod white_mod {
    fn white() {}
}
"#;
        // Notice that the empty line is not visible, but there's a trailing newline
        // from the "attrs" if you see the final output carefully. 
        assert_eq!(output, expected);
    }

    /// 8) If items contain multiple lines, each line is prefixed with 4 spaces inside the mod.
    #[test]
    fn test_multi_line_item_indentation() {
        let mut module = ModuleInterface::new(None, None, "lines_mod".to_string());
        // This item has line breaks
        let item_content = "/// item doc line\npub fn multiline() {\n    // body\n}";
        module.add_item(ConsolidatedItem::MockTest(item_content.to_string()));

        let output = format!("{}", module);
        let expected = r#"mod lines_mod {
    /// item doc line
    pub fn multiline() {
        // body
    }
}
"#;
        assert_eq!(
            output, expected,
            "Each line in the item should be indented by 4 spaces"
        );
    }
}
