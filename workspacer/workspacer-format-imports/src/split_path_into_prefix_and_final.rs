// ---------------- [ File: src/split_path_into_prefix_and_final.rs ]
crate::ix!();

/// Splits "std::collections::{HashMap,HashSet}" or "std::collections::HashMap"
pub fn split_path_into_prefix_and_final(full_path: &str) -> (String, Vec<String>) {
    if let Some(idx) = full_path.find("::{") {
        let prefix = &full_path[..idx];
        let inside = &full_path[idx+3..full_path.len()-1]; // skip "::{...}"
        let segments = inside
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        (prefix.trim().to_string(), segments)
    } else {
        if let Some(idx2) = full_path.rfind("::") {
            let prefix = &full_path[..idx2];
            let final_seg = &full_path[idx2+2..];
            (prefix.trim().to_string(), vec![final_seg.trim().to_string()])
        } else {
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
        let input = "std::collections::{HashMap,HashSet}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals.len(), 2);
        assert!(finals.contains(&"HashMap".to_string()));
        assert!(finals.contains(&"HashSet".to_string()));
    }

    /// 2) If there is spacing around the final braces and commas, we trim them.
    ///    E.g. "std::collections::{  HashMap ,  HashSet }"
    #[test]
    fn test_braced_with_spaces() {
        let input = "std::collections::{  HashMap ,  HashSet }";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals, vec!["HashMap", "HashSet"]);
    }

    /// 3) If there's only one item in the braces, we still parse it into a single-element finals vec.
    #[test]
    fn test_braced_single_item() {
        let input = "my::mod::{Single}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        assert_eq!(prefix, "my::mod");
        assert_eq!(finals, vec!["Single"]);
    }

    /// 4) If the path does not contain "::{", but has "::" => we split at the last "::"
    ///    E.g. "std::collections::HashMap" => prefix="std::collections", finals=["HashMap"].
    #[test]
    fn test_colon_colon_without_braces() {
        let input = "std::collections::HashMap";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals, vec!["HashMap"]);
    }

    /// 5) If there are multiple "::" but no "::{", we only split at the *last* occurrence.
    ///    E.g. "a::b::c" => prefix="a::b", finals=["c"].
    #[test]
    fn test_multiple_double_colons_no_braces() {
        let input = "a::b::c";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        assert_eq!(prefix, "a::b");
        assert_eq!(finals, vec!["c"]);
    }

    /// 6) If there is no "::" at all => prefix="", finals=[the entire input].
    ///    E.g. "std" => prefix="", finals=["std"].
    #[test]
    fn test_no_colons_at_all() {
        let input = "std";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        assert_eq!(prefix, "");
        assert_eq!(finals, vec!["std"]);
    }

    /// 7) Extra spacing around the final segment => we trim it.
    #[test]
    fn test_spacing_in_final_segment() {
        let input = "alpha::  final   ";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        assert_eq!(prefix, "alpha");
        assert_eq!(finals, vec!["final"]);
    }

    /// 8) If there's only "::" but no trailing segment => final is "" (after trim).
    ///    e.g. "std::"
    #[test]
    fn test_trailing_double_colon() {
        let input = "std::";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        // The code tries to parse the last segment => empty string
        assert_eq!(prefix, "std");
        assert_eq!(finals, vec![""], "Final is empty string");
    }

    /// 9) If the braces are empty => "std::{ }" => finals should be an empty string inside or no segments?
    ///    We'll see how the code behaves. The inside would be "" => results in an empty final?
    #[test]
    fn test_empty_braces() {
        let input = "std::collections::{}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        // prefix => "std::collections", finals => [""]
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals.len(), 1);
        assert_eq!(finals[0], "");
    }

    /// 10) If there's whitespace around braces => e.g. "std::collections:: {HashMap}" => 
    ///    we won't find "::{", so we treat it as a normal last "::"? 
    ///    => prefix="std::collections:: {HashMap" is weird, but let's see.
    #[test]
    fn test_weird_spacing_breaks_brace_detection() {
        let input = "std::collections:: {HashMap}";
        let (prefix, finals) = split_path_into_prefix_and_final(input);
        // We'll not find "::{", so we do the rfind("::") => prefix="std::collections:: {HashMap"? 
        // Actually the code is naive => index of last "::" is around "collections::", 
        // so prefix="std::collections", final=" {HashMap}"
        // The final is " {HashMap}", trimmed => "{HashMap}"
        assert_eq!(prefix, "std::collections");
        assert_eq!(finals, vec!["{HashMap}"]);
    }
}
