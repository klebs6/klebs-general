crate::ix!();

// -------------------------------------------------------------------------
// 5) Remove old use statements from old_text, preserving non-use lines
// -------------------------------------------------------------------------
pub fn remove_old_use_statements(uses_data: &[UseItemInfo], old_text: &str) -> String {
    let mut out = String::new();
    let mut start_pos = 0usize;

    for ud in uses_data {
        let s = *ud.range_start();
        let e = *ud.range_end();

        if s > start_pos {
            out.push_str(&old_text[start_pos..s]);
        }
        start_pos = e;
    }
    if start_pos < old_text.len() {
        out.push_str(&old_text[start_pos..]);
    }

    out
}

#[cfg(test)]
mod test_remove_old_use_statements {
    use super::{remove_old_use_statements, UseItemInfoBuilder};

    /// 1) If no uses => returns old_text unchanged
    #[test]
    fn test_empty_uses_data() {
        let old_text = "fn main() {}";
        let uses_data = vec![];
        let out = remove_old_use_statements(&uses_data, old_text);
        assert_eq!(out, old_text);
    }

    /// 2) If one use statement => remove that range
    #[test]
    fn test_single_use_removed() {
        let old_text = r#"
fn main() {}
pub(crate) use std::io;
"#;
        // Suppose the use spans lines 2..3. We'll guess the range
        let info = UseItemInfoBuilder::default()
            .leading_comments(vec![])
            .raw_text("".to_string())
            .range_start(15_usize) // approximate
            .range_end(40_usize)
            .visibility("pub(crate)".to_string())
            .path_list("std::io".to_string())
            .build().unwrap();

        let out = remove_old_use_statements(&[info], old_text);
        // Ensure the snippet for lines 2..3 is gone
        assert!(out.contains("fn main()"));
        assert!(!out.contains("use std::io"));
    }

    // etc...
}
