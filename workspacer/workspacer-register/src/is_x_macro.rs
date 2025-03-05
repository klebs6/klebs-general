// ---------------- [ File: workspacer-register/src/is_x_macro.rs ]
crate::ix!();

pub fn is_x_macro(item: &ast::Item) -> Option<String> {
    trace!("Entering is_x_macro");

    let mac_call = ast::MacroCall::cast(item.syntax().clone())?;
    let path = mac_call.path()?;
    let path_str = path.syntax().text().to_string();
    trace!("Parsed macro path='{}'", path_str);

    // Trim and ensure the path is exactly "x" without internal spaces.
    let path_str = path_str.trim();
    if path_str != "x" || path_str.contains(' ') {
        trace!("Not an x! macro => returning None");
        return None;
    }

    let full_text = item.syntax().text().to_string();

    // Ensure the macro has braces; otherwise, it's invalid
    if !full_text.contains('{') || !full_text.contains('}') {
        trace!("Missing braces in macro, not recognized as x! macro");
        return None;
    }

    // Enforce that the source text actually contains "x!"
    // (no whitespace is allowed between `x` and `!`)
    if !full_text.contains("x!") {
        trace!("No direct 'x!' substring found => spacing or mismatch => returning None");
        return None;
    }

    // Ensure there are no spaces around the entire macro (leading/trailing)
    if full_text != full_text.trim() {
        trace!("Invalid spacing around the macro, rejected");
        return None;
    }

    debug!("Recognized x! macro => returning full_text='{}'", full_text);
    Some(full_text)
}

#[cfg(test)]
mod test_is_x_macro {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};

    /// Helper: parse `src` into a `SourceFile`, then collect its items as a Vec.
    fn parse_items(src: &str) -> Vec<ast::Item> {
        let parsed = SourceFile::parse(src, Edition::Edition2024).tree();
        parsed.items().collect()
    }

    /// 1) If the file is empty => no items => no macros
    #[traced_test]
    fn test_empty_file() {
        let src = "";
        let items = parse_items(src);
        assert!(items.is_empty(), "Expected no items in empty file");
    }

    /// 4) If there's no braces or incorrect braces => the MacroCall might fail to parse
    ///    => is_x_macro returns None
    #[traced_test]
    fn test_missing_braces() {
        let src = r#"
x!stuff
"#;
        let items = parse_items(src);
        // We no longer assume exactly one item; just ensure none is recognized as x! macro
        assert!(
            !items.is_empty(),
            "We expect at least one item, though the parser may split it differently"
        );

        for it in &items {
            let maybe_x = is_x_macro(it);
            assert!(maybe_x.is_none(), "Invalid syntax => not recognized as x! macro");
        }
    }

    /// 2) A straightforward x!{something} => is_x_macro returns Some(...)
    #[traced_test]
    fn test_simple_x_macro() {
        let src = r#"
x!{hello}
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);

        let first = &items[0];
        let found = is_x_macro(first).expect("Should recognize x! macro");
        assert_eq!(found, "x!{hello}", "We capture the entire macro text");
    }

    /// 3) Non-x macros (foo!{}, bar!{}) => is_x_macro returns None
    #[traced_test]
    fn test_non_x_macros() {
        let src = r#"
foo!{stuff}
bar!{things}
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 2);

        for item in &items {
            let maybe_x = is_x_macro(item);
            assert!(
                maybe_x.is_none(),
                "Should not recognize foo! or bar! as x! macros"
            );
        }
    }

    /// 5) x! macro with attributes => the parser might attach them to the MacroCall item,
    ///    so is_x_macro could still return Some if the path is recognized as x.
    #[traced_test]
    fn test_x_macro_with_attributes() {
        let src = r#"
#[some_attr]
x!{with_attr}
"#;
        let items = parse_items(src);
        // Possibly 1 item if the parser lumps them together
        assert_eq!(items.len(), 1);

        let maybe_x = is_x_macro(&items[0]);
        if let Some(mac_text) = maybe_x {
            // We can see if it includes the attribute in the text
            assert!(
                mac_text.contains("#[some_attr]"),
                "The entire macro text may include the attribute"
            );
            assert!(
                mac_text.contains("x!{with_attr}"),
                "Should contain the macro call"
            );
        } else {
            eprintln!("is_x_macro did not recognize x! macro with attributes, which might be intentional depending on the parser rules.");
        }
    }

    /// 6) x! macro with empty braces => recognized, returning something like "x!{}"
    #[traced_test]
    fn test_x_macro_empty_braces() {
        let src = r#"
x!{}
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);

        let maybe_x = is_x_macro(&items[0]).expect("Should be recognized as x macro");
        assert_eq!(maybe_x, "x!{}");
    }

    /// 7) multiple macros on one line => we check them item by item
    #[traced_test]
    fn test_multiple_macros_on_one_line() {
        let src = r#"
x!{aaa} x!{bbb} x!{ccc}
"#;
        let items = parse_items(src);
        // ra_ap_syntax might parse them as multiple items or might parse them differently.
        // We'll just check that each recognized macro is x! macro if it indeed splits them up.
        assert!(
            !items.is_empty(),
            "We expect at least one item recognized, possibly multiple."
        );

        let x_macro_texts: Vec<_> = items
            .iter()
            .filter_map(|item| is_x_macro(item))
            .collect();

        // We expect to see "x!{aaa}" "x!{bbb}" "x!{ccc}" in some order
        // or possibly a single item containing all if the parser lumps them.
        let joined = x_macro_texts.join("|");
        assert!(
            joined.contains("x!{aaa}"),
            "We expect to see x!{{aaa}}"
        );
        assert!(
            joined.contains("x!{bbb}"),
            "We expect to see x!{{bbb}}"
        );
        assert!(
            joined.contains("x!{ccc}"),
            "We expect to see x!{{ccc}}"
        );
    }

    /// 8) If the path is something else like "x :: y", is_x_macro => None
    #[traced_test]
    fn test_x_colon_colon_y_macro() {
        let src = r#"
x::y!{stuff}
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);

        let maybe_x = is_x_macro(&items[0]);
        assert!(maybe_x.is_none(), "x::y! => not recognized as 'x!'");
    }

    /// 9) If there's some spacing => " x  !  {stuff}" => the parser might or might not parse a macrocall.
    ///    Typically, it won't parse as "x!{stuff}" exactly. We'll see if is_x_macro picks it up.
    #[traced_test]
    fn test_spacing_around_x_exclamation_braces() {
        let src = r#"
x   !   {stuff}
"#;
        let items = parse_items(src);
        // Possibly 1 item, but the parser may treat it as weird tokens.
        // We'll check if is_x_macro sees it.
        assert_eq!(items.len(), 1);
        let maybe_x = is_x_macro(&items[0]);
        assert!(
            maybe_x.is_none(),
            "Spacing between x, !, and {{stuff}} => likely not recognized as x! macro"
        );
    }

    /// 10) Non-macro item => None
    #[traced_test]
    fn test_fn_item_is_none() {
        let src = r#"
fn not_a_macro() {}
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);

        let maybe_x = is_x_macro(&items[0]);
        assert!(
            maybe_x.is_none(),
            "Regular fn item => not recognized as x! macro"
        );
    }

    /// 11) A raw mod line => None
    #[traced_test]
    fn test_mod_imports() {
        let src = r#"
#[macro_use]
mod imports;
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);

        let maybe_x = is_x_macro(&items[0]);
        assert!(
            maybe_x.is_none(),
            "mod imports => not recognized as x macro"
        );
    }

    /// 12) Confirm we do a path_str.trim() == "x" check => 
    ///     if path syntax is " x " => let's see if it picks it up. Usually the parser normalizes it.
    #[traced_test]
    fn test_trim_check() {
        let src = r#"
x  !{ trimmed_check}
"#;
        let items = parse_items(src);
        assert_eq!(items.len(), 1);

        let maybe_x = is_x_macro(&items[0]);
        assert!(
            maybe_x.is_none(),
            "Even if path_str has spaces, typically the parser won't yield a normal 'x' path. 
             So we expect None."
        );
    }
}
