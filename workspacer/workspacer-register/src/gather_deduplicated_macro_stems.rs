// ---------------- [ File: workspacer-register/src/gather_deduplicated_macro_stems.rs ]
crate::ix!();

pub fn gather_deduplicated_macro_stems(
    old_macros: &[ExistingXMacro],
    new_top_block: &str,
) -> Vec<String> {
    trace!("Entering gather_deduplicated_macro_stems");

    fn extract_stem(x_macro_text: &str) -> Option<String> {
        let start = x_macro_text.find('{')?;
        let end   = x_macro_text.rfind('}')?;
        if start + 1 <= end {
            Some(x_macro_text[start+1..end].trim().to_string())
        } else {
            None
        }
    }

    // old stems
    let mut stems: Vec<String> = old_macros
        .iter()
        .filter_map(|em| {
            let s = extract_stem(em.text());
            trace!("From old macro text='{}' => stem={:?}", em.text(), s);
            s
        })
        .collect();

    // parse macros out of new_top_block
    let parsed_newblock = SourceFile::parse(new_top_block, Edition::Edition2021).tree();
    for item in parsed_newblock.items() {
        if let Some(txt) = is_x_macro(&item) {
            if let Some(stm) = extract_stem(&txt) {
                debug!("Found new top_block macro with stem='{}'", stm);
                stems.push(stm);
            } else {
                trace!("Unable to parse stem from macro text='{}'", txt);
            }
        }
    }

    stems.sort();
    stems.dedup();
    debug!("Final deduplicated stems: {:?}", stems);

    trace!("Exiting gather_deduplicated_macro_stems");
    stems
}

#[cfg(test)]
mod test_gather_deduplicated_macro_stems {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};

    /// A small helper to build an `ExistingXMacro` easily.
    /// We'll assume offset is not super critical for these tests, so we'll pass 0..0 as a dummy range.
    fn make_existing_x_macro(text: &str) -> ExistingXMacro {
        ExistingXMacroBuilder::default()
            .text(text.to_string())
            // We'll cheat here:  "range(0..0)" so we don't have to parse real offsets for these unit tests.
            .range(TextRange::new(TextSize::from(0), TextSize::from(0)))
            .build()
            .unwrap()
    }

    /// A convenience function to parse `new_top_block` into a SourceFile,
    /// just to ensure `is_x_macro` is recognized. We won't do heavy offset checks.
    fn parse_block(block: &str) -> SourceFile {
        SourceFile::parse(block, Edition::Edition2021).tree()
    }

    /// 1) If there are no old macros and the `new_top_block` has no macros => empty result
    #[traced_test]
    fn test_no_old_no_new() {
        let old_macros = vec![];
        let new_top_block = r#"
// no macros here
fn something() {}
"#;
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        assert!(result.is_empty(), "No macros => no stems");
    }

    /// 2) If there are old macros only => we get their stems
    #[traced_test]
    fn test_old_macros_only() {
        let old_macros = vec![
            make_existing_x_macro("x!{alpha}"),
            make_existing_x_macro("x!{beta}"),
        ];
        let new_top_block = "// no macros in new block";
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        assert_eq!(result, vec!["alpha", "beta"], "Should gather old stems only");
    }

    /// 3) If there are new macros only => we get their stems
    #[traced_test]
    fn test_new_macros_only() {
        let old_macros = vec![];
        let new_top_block = r#"
            // here's a macro
            x!{gamma}
            x!{delta}
        "#;
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        assert_eq!(result, vec!["delta", "gamma"], "Should gather new stems, sorted");
    }

    /// 4) If both old and new macros appear => unify + deduplicate + sort
    #[traced_test]
    fn test_both_old_and_new() {
        let old_macros = vec![
            make_existing_x_macro("x!{alpha}"),
            make_existing_x_macro("x!{beta}"),
        ];
        let new_top_block = r#"
            x!{beta}
            x!{gamma}
        "#;
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        // alpha, beta, gamma => sorted => alpha, beta, gamma
        assert_eq!(result, vec!["alpha", "beta", "gamma"]);
    }

    /// 5) Duplicate macros in both old and new => ensure they appear only once
    #[traced_test]
    fn test_duplicates() {
        let old_macros = vec![
            make_existing_x_macro("x!{dup}"),
            make_existing_x_macro("x!{unique_old}"),
        ];
        let new_top_block = r#"
            x!{dup} // also appears here
            x!{unique_new}
        "#;
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        // combined: dup, unique_old, dup, unique_new => after dedup + sort => dup, unique_new, unique_old => 
        // but wait, alphabetical => dup, unique_new, unique_old
        // but let's see: "unique_new" < "unique_old"? Actually "unique_new" < "unique_old"? 
        // alphabetical by ASCII => "dup", "unique_new", "unique_old"
        // "unique_new" < "unique_old"? yes, because 'n' < 'o'.
        assert_eq!(result, vec!["dup", "unique_new", "unique_old"]);
    }

    /// 6) Old macros might have weird spacing => e.g. "x!{   spaced }" => the extracted stem is "spaced"
    #[traced_test]
    fn test_stem_extraction_trims() {
        let old_macros = vec![
            make_existing_x_macro("x!{   spaced    }"),
        ];
        let new_top_block = "";
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        assert_eq!(result, vec!["spaced"]);
    }

    /// 7) new_top_block might have macros with spacing => confirm we parse the stems
    #[traced_test]
    fn test_new_block_stems_with_whitespace() {
        let old_macros = vec![];
        let new_top_block = r#"
            x!{   one  }
            x!{    two   }
            x!{  three    }
        "#;
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        // sorted => ["one", "three", "two"]
        assert_eq!(result, vec!["one", "three", "two"]);
    }

    /// 8) Macros with empty braces => stem is ""
    #[traced_test]
    fn test_empty_braces() {
        let old_macros = vec![
            make_existing_x_macro("x!{}"),
            make_existing_x_macro("x!{   }"),
        ];
        let new_top_block = r#"
            x!{}
        "#;

        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);
        // We have multiple empty stems => dedup => just "" (empty string).
        assert_eq!(result, vec![""]);
    }

    /// 9) Non-x macros in new_top_block => no stems from them
    #[traced_test]
    fn test_non_x_macros_in_new_top_block_ignored() {
        let old_macros = vec![
            make_existing_x_macro("x!{old_stem}"),
        ];
        let new_top_block = r#"
            foo!{some_stem}
            x!{another_x}
        "#;
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        // We gather "old_stem" from old macros, plus "another_x" from new, ignoring foo! macro.
        // sorted => "another_x", "old_stem"
        assert_eq!(result, vec!["another_x", "old_stem"]);
    }

    /// 10) Some scenario with random text that looks like braces but not x! => no stems
    #[traced_test]
    fn test_random_braces_not_macro() {
        let old_macros = vec![
            make_existing_x_macro("x!{valid}"),
        ];
        let new_top_block = r#"
            x ! { maybe } ?

            x!(not quite)
        "#;
        // parse => likely no recognized x! macros because `is_x_macro` demands "x!{"
        let result = gather_deduplicated_macro_stems(&old_macros, new_top_block);

        // only old macro => "valid"
        assert_eq!(result, vec!["valid"]);
    }
}
