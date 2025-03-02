// ---------------- [ File: src/split_path_into_prefix_and_final.rs ]
crate::ix!();

pub fn split_path_into_prefix_and_final(full_path: &str) -> (String, Vec<String>) {
    info!(
        "split_path_into_prefix_and_final => entered with full_path={:?}",
        full_path
    );

    if let Some(idx) = full_path.find("::{") {
        debug!("Detected braced path usage at index {}", idx);
        let prefix = &full_path[..idx];
        let inside = &full_path[idx + 3..full_path.len() - 1]; // skip "::{...}"
        let segments: Vec<_> = inside
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        debug!("Returning prefix={:?} with {} finals from braces", prefix, segments.len());
        return (prefix.trim().to_string(), segments);
    } else {
        if let Some(idx2) = full_path.rfind("::") {
            debug!("Found last '::' at index {}", idx2);
            let prefix = &full_path[..idx2];
            let final_seg = &full_path[idx2 + 2..];
            let pr = prefix.trim().to_string();
            let fs = final_seg.trim().to_string();
            debug!("Returning prefix={:?}, single final={:?}", pr, fs);
            (pr, vec![fs])
        } else {
            debug!("No '::' found => entire input is final");
            ("".to_string(), vec![full_path.trim().to_string()])
        }
    }
}

#[cfg(test)]
mod test_split_path_into_prefix_and_final {
    use super::*;

    /// 1) If the input contains "::{", we parse the prefix up to that
    ///    and split the segments inside braces by commas.
    ///    E.g. "std::collections::{HashMap,HashSet}" => prefix="std::collections", finals=["HashMap","HashSet"].
    #[test]
    fn test_braced_multiple_segments() {
        info!("test_braced_multiple_segments => start");
        let input = "std::collections::{HashMap,HashSet}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals.len(), 2);
        assert!(finals.contains(&"HashMap".to_string()));
        assert!(finals.contains(&"HashSet".to_string()));
        info!("test_braced_multiple_segments => done");
    }

    /// 2) If there is spacing around the final braces and commas, we trim them.
    #[test]
    fn test_braced_with_spaces() {
        info!("test_braced_with_spaces => start");
        let input = "std::collections::{  HashMap ,  HashSet }";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals, vec!["HashMap", "HashSet"]);
        info!("test_braced_with_spaces => done");
    }

    /// 3) If there's only one item in the braces, we still parse it into a single-element finals vec.
    #[test]
    fn test_braced_single_item() {
        info!("test_braced_single_item => start");
        let input = "my::mod::{Single}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "my::mod");
        assert_eq!(finals, vec!["Single"]);
        info!("test_braced_single_item => done");
    }

    /// 4) If the path does not contain "::{", but has "::" => we split at the last "::"
    #[test]
    fn test_colon_colon_without_braces() {
        info!("test_colon_colon_without_braces => start");
        let input = "std::collections::HashMap";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals, vec!["HashMap"]);
        info!("test_colon_colon_without_braces => done");
    }

    /// 5) Multiple "::" => we only split at the last occurrence.
    #[test]
    fn test_multiple_double_colons_no_braces() {
        info!("test_multiple_double_colons_no_braces => start");
        let input = "a::b::c";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "a::b");
        assert_eq!(finals, vec!["c"]);
        info!("test_multiple_double_colons_no_braces => done");
    }

    /// 6) No "::" => prefix="", finals=[the entire input].
    #[test]
    fn test_no_colons_at_all() {
        info!("test_no_colons_at_all => start");
        let input = "std";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "");
        assert_eq!(finals, vec!["std"]);
        info!("test_no_colons_at_all => done");
    }

    /// 7) Extra spacing around the final segment => we trim it.
    #[test]
    fn test_spacing_in_final_segment() {
        info!("test_spacing_in_final_segment => start");
        let input = "alpha::  final   ";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "alpha");
        assert_eq!(finals, vec!["final"]);
        info!("test_spacing_in_final_segment => done");
    }

    /// 8) If there's only "::" but no trailing segment => final is "" (after trim).
    #[test]
    fn test_trailing_double_colon() {
        info!("test_trailing_double_colon => start");
        let input = "std::";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "std");
        assert_eq!(finals, vec![""], "Final is empty string");
        info!("test_trailing_double_colon => done");
    }

    /// 9) If the braces are empty => "std::{ }" => finals might be empty or [""]
    #[test]
    fn test_empty_braces() {
        info!("test_empty_braces => start");
        let input = "std::collections::{}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals.len(), 1);
        assert_eq!(finals[0], "");
        info!("test_empty_braces => done");
    }

    /// 10) "std::collections:: {HashMap}" => won't find "::{"
    #[test]
    fn test_weird_spacing_breaks_brace_detection() {
        info!("test_weird_spacing_breaks_brace_detection => start");
        let input = "std::collections:: {HashMap}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        debug!("prefix={:?}, finals={:?}", prefix, finals);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals, vec!["{HashMap}"]);
        info!("test_weird_spacing_breaks_brace_detection => done");
    }
}
