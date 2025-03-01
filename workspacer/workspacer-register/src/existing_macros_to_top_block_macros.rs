// ---------------- [ File: src/existing_macros_to_top_block_macros.rs ]
crate::ix!();

pub fn existing_macros_to_top_block_macros(old_macros: &[ExistingXMacro]) -> Vec<TopBlockMacro> {
    let mut out = vec![];
    for em in old_macros {
        if let Some(stem) = extract_stem(em.text()) {
            // Transform empty/whitespace-only comments into None
            let leading = match em.leading_comments() {
                Some(s) => {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(s.clone())
                    }
                }
                None => None,
            };

            let macr = TopBlockMacroBuilder::default()
                .stem(stem)
                .leading_comments(leading)
                .build()
                .unwrap();

            out.push(macr);
        }
    }
    out
}

fn extract_stem(full_text: &str) -> Option<String> {
    let start = full_text.find('{')?;
    let end   = full_text.rfind('}')?;
    Some(full_text[start+1..end].trim().to_string())
}

#[cfg(test)]
mod test_existing_macros_to_top_block_macros {
    use super::*;
    use tracing::{trace, debug};

    fn ex_mac(text: &str, comments: &str) -> ExistingXMacro {
        ExistingXMacroBuilder::default()
            .text(text)
            .range(TextRange::new(TextSize::from(0), TextSize::from(text.len() as u32)))
            .leading_comments(Some(comments.to_string()))
            .build()
            .unwrap()
    }

    /// 1) empty => returns empty
    #[traced_test]
    fn test_empty_list() {
        trace!("Starting test_empty_list for existing_macros_to_top_block_macros");
        let result = existing_macros_to_top_block_macros(&[]);
        debug!("Result = {:?}", result);
        assert!(result.is_empty(), "No old macros => empty vector");
    }

    /// 2) Single old macro => parse the stem
    #[traced_test]
    fn test_single_macro() {
        trace!("Starting test_single_macro for existing_macros_to_top_block_macros");
        let old = [ex_mac("x!{alpha}", "")];
        let result = existing_macros_to_top_block_macros(&old);
        debug!("Result = {:?}", result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "alpha", "Expected to parse 'alpha'");
        assert!(result[0].leading_comments().is_none(), "No leading comments given");
    }

    /// 3) Leading comments => carried over
    #[traced_test]
    fn test_leading_comments_carried_over() {
        trace!("Starting test_leading_comments_carried_over for existing_macros_to_top_block_macros");
        let old = [ex_mac("x!{beta}", "// doc line\n")];
        let result = existing_macros_to_top_block_macros(&old);
        debug!("Result = {:?}", result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "beta");
        assert!(result[0].leading_comments().as_ref().unwrap().contains("doc line"), "Should preserve leading comment");
    }

    /// 4) If we cannot parse the stem => skip that macro
    #[traced_test]
    fn test_unparsable_macro_skipped() {
        trace!("Starting test_unparsable_macro_skipped for existing_macros_to_top_block_macros");
        let old = [
            ex_mac("x! alpha", ""),    // missing braces => skip
            ex_mac("x!{delta}", ""),   // valid
            ex_mac("x!{}", ""),        // empty braces => valid with empty stem
            ex_mac("junk text", ""),   // not recognized => skip
        ];
        let result = existing_macros_to_top_block_macros(&old);
        debug!("Result = {:?}", result);

        // Only "x!{delta}" => "delta" and "x!{}" => "" remain
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].stem(), "delta");
        assert_eq!(result[1].stem(), "");
    }
}
