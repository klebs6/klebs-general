// ---------------- [ File: src/is_imports_line.rs ]
crate::ix!();

pub fn is_imports_line(item: &ast::Item) -> bool {
    trace!("Entering is_imports_line");

    if let Some(module_item) = ast::Module::cast(item.syntax().clone()) {
        if let Some(name_ident) = module_item.name() {
            let name_txt = name_ident.text();
            debug!("Module name='{}'", name_txt);
            let is_imports = name_txt == "imports";
            trace!("Returning {}", is_imports);
            return is_imports;
        }
    } else if let Some(use_item) = ast::Use::cast(item.syntax().clone()) {
        let text = use_item.syntax().text().to_string();
        trace!("Found a use item => text='{}'", text);
        if text.contains("imports::*") {
            debug!("This use line references imports::* => returning true");
            return true;
        }
    }

    trace!("Not recognized as an imports line => returning false");
    false
}

#[cfg(test)]
mod test_is_imports_line {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};

    /// A quick helper that parses the entire file as a `SourceFile`
    /// and then returns the `ast::Item`s in the order they appear.
    fn parse_items(src: &str) -> Vec<ast::Item> {
        let parsed = SourceFile::parse(src, Edition::Edition2021).tree();
        parsed.items().collect()
    }

    /// 1) Empty file => no items => we won't find any `imports` lines
    #[traced_test]
    fn test_empty_file() {
        let src = "";
        let items = parse_items(src);
        assert!(items.is_empty(), "No items in empty file");
        // There's nothing to check, but we confirm no is_imports_line calls would return true.
    }

    /// 2) A simple `mod imports;` => name_ident is "imports" => should return true
    #[traced_test]
    fn test_mod_imports() {
        let src = r#"
mod imports;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1, "We expect exactly one item (mod imports)");
        let first = &items[0];
        assert!(
            is_imports_line(first),
            "mod imports; => should be recognized as an imports line"
        );
    }

    /// 3) A `#[macro_use] mod imports;` => also recognized as an imports line
    #[traced_test]
    fn test_macro_use_mod_imports() {
        let src = r#"
#[macro_use]
mod imports;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);
        let first = &items[0];
        assert!(
            is_imports_line(first),
            "#[macro_use] mod imports; => recognized as imports line"
        );
    }

    /// 4) A `use imports::*;` => recognized, because the text contains "imports::*"
    #[traced_test]
    fn test_use_imports_star() {
        let src = r#"
use imports::*;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);
        let first = &items[0];
        assert!(
            is_imports_line(first),
            "use imports::*; => recognized as an imports line"
        );
    }

    /// 5) A more complex combination: `#[macro_use] mod imports; use imports::*;`
    ///    both might appear as a single item or multiple items, depending how parser sees it.
    #[traced_test]
    fn test_macro_use_mod_imports_plus_use_imports_star() {
        let src = r#"
#[macro_use] mod imports; use imports::*;
"#;
        let items = parse_items(src);

        // This might parse as two items or just one item, depending on how ra_ap_syntax splits them,
        // but let's see. We'll check each item for is_imports_line.
        assert!(!items.is_empty(), "Should parse at least one item");
        let any_imports = items.iter().any(|item| is_imports_line(item));
        assert!(
            any_imports,
            "At least one of the items should be recognized as an imports line"
        );
    }

    /// 6) A different mod name => false
    #[traced_test]
    fn test_mod_something_else() {
        let src = r#"
mod something_else;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);
        let first = &items[0];
        assert!(
            !is_imports_line(first),
            "mod something_else => not recognized as an imports line"
        );
    }

    /// 7) A `use foo::imports;` => does not contain "imports::*", so we do not treat it as an imports line
    #[traced_test]
    fn test_use_foo_imports_not_star() {
        let src = r#"
use foo::imports;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);
        let first = &items[0];
        assert!(
            !is_imports_line(first),
            "use foo::imports => does NOT match 'imports::*' => false"
        );
    }

    /// 8) If the item is `mod importsXYZ;`, name_ident is "importsXYZ" => not "imports"
    #[traced_test]
    fn test_mod_importsxyz() {
        let src = r#"
mod importsxyz;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);
        let first = &items[0];
        assert!(
            !is_imports_line(first),
            "mod importsxyz => not recognized as 'imports' exactly"
        );
    }

    /// 9) A more complex `use self::imports::sub;` => we check if `.text()` contains "imports::*"
    ///    It does not, so we do NOT treat it as an imports line.
    #[traced_test]
    fn test_use_sub_imports_no_star() {
        let src = r#"
use self::imports::sub;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);
        let first = &items[0];
        assert!(
            !is_imports_line(first),
            "use self::imports::sub; => not containing 'imports::*' => false"
        );
    }

    /// 10) If the use statement is uppercase or partial => we don't match "imports::*"
    #[traced_test]
    fn test_use_partial_caps() {
        let src = r#"
use Imports::*;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);

        let first = &items[0];
        assert!(
            !is_imports_line(first),
            "use Imports::* => not recognized because text.contains('imports::*') fails"
        );
    }

    /// 11) multiple items => we check each. 
    ///     We'll confirm we find exactly the ones that match mod imports or use imports::* as true.
    #[traced_test]
    fn test_multiple_items_mixed() {
        let src = r#"
mod something;
#[macro_use] mod imports;
fn foo() {}
use imports::*;
use bar::*;
"#;
        let items = parse_items(src);
        // items might parse as 5 or so. We'll check each for is_imports_line
        // We'll count how many are recognized as "imports line".
        let recognized_count = items.iter().filter(|item| is_imports_line(item)).count();
        // "mod something" => false
        // "#[macro_use] mod imports;" => true
        // "fn foo()" => false
        // "use imports::*" => true
        // "use bar::*" => false
        assert_eq!(recognized_count, 2, "We expect exactly 2 recognized imports lines");
    }
}
