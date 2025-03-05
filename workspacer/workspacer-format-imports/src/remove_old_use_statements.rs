// ---------------- [ File: workspacer-format-imports/src/remove_old_use_statements.rs ]
crate::ix!();

pub fn remove_old_use_statements(uses_data: &[UseItemInfo], old_text: &str) -> String {
    info!("remove_old_use_statements => start");
    debug!("old_text length={}", old_text.len());
    debug!("use_data count={}", uses_data.len());

    let mut out = String::new();
    let mut start_pos = 0usize;

    for ud in uses_data {
        trace!("Removing range {}..{} for path_list={:?}", ud.range_start(), ud.range_end(), ud.path_list());
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

    debug!(
        "remove_old_use_statements => returning text length={}",
        out.len()
    );
    info!("remove_old_use_statements => done");
    out
}

#[cfg(test)]
mod test_remove_old_use_statements {
    use super::*;

    /// 1) If no uses => returns old_text unchanged
    #[test]
    fn test_empty_uses_data() {
        info!("test_empty_uses_data => start");
        let old_text = "fn main() {}";
        let uses_data = vec![];
        let out = remove_old_use_statements(&uses_data, old_text);
        debug!("Result => length={}, content=\n{}", out.len(), out);
        assert_eq!(out, old_text);
        info!("test_empty_uses_data => done");
    }

    /// 2) If one use statement => remove that range
    #[test]
    fn test_single_use_removed() {
        info!("test_single_use_removed => start");
        let old_text = r#"
fn main() {}
pub(crate) use std::io;
"#;
        let info = UseItemInfoBuilder::default()
            .leading_comments(vec![])
            .raw_text("".to_string())
            .range_start(15_usize)
            .range_end(40_usize)
            .visibility("pub(crate)".to_string())
            .path_list("std::io".to_string())
            .trailing_comment(None)
            .build()
            .unwrap();

        debug!("Removing single use => range=15..40");
        let out = remove_old_use_statements(&[info], old_text);
        debug!("Result => length={}, content=\n{}", out.len(), out);

        assert!(out.contains("fn main()"));
        assert!(!out.contains("use std::io"));
        info!("test_single_use_removed => done");
    }
}
