crate::ix!();

// -------------------------------------------------------------------------
// 3) Group them by (visibility + prefix), and unify final segments
// -------------------------------------------------------------------------
pub fn group_and_sort_uses(
    uses_data: &[UseItemInfo]
) -> (
    BTreeMap<(String, String), BTreeSet<String>>,
    BTreeMap<(String, String), Vec<String>>
) {
    let mut grouped_map: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut comment_map: BTreeMap<(String,String), Vec<String>> = BTreeMap::new();

    for ud in uses_data {
        let (prefix, finals) = split_path_into_prefix_and_final(ud.path_list());
        let key = (ud.visibility().clone(), prefix.clone());

        grouped_map.entry(key.clone()).or_default().extend(finals);
        comment_map.entry(key).or_default().extend(ud.leading_comments().clone());
    }

    (grouped_map, comment_map)
}

#[cfg(test)]
mod test_group_and_sort_uses {
    use super::{group_and_sort_uses, UseItemInfoBuilder};
    use std::collections::BTreeMap;

    /// 1) If we pass an empty list => returns empty maps.
    #[test]
    fn test_empty_list() {
        let uses_data = vec![];
        let (grouped_map, comment_map) = group_and_sort_uses(&uses_data);
        assert!(grouped_map.is_empty());
        assert!(comment_map.is_empty());
    }

    /// 2) Two `use` items with same prefix => they merge.
    #[test]
    fn test_merge_same_prefix() {
        let item1 = UseItemInfoBuilder::default()
            .visibility("pub(crate)".to_string())
            .path_list("std::collections::HashMap".to_string())
            .leading_comments(vec!["// comment1\n".to_string()])
            .raw_text("".to_string())
            .range_start(0_usize)
            .range_end(0_usize)
            .build().unwrap();

        let item2 = UseItemInfoBuilder::default()
            .visibility("pub(crate)".to_string())
            .path_list("std::collections::HashSet".to_string())
            .leading_comments(vec!["// comment2\n".to_string()])
            .raw_text("".to_string())
            .range_start(0_usize)
            .range_end(0_usize)
            .build().unwrap();

        let uses_data = vec![item1, item2];
        let (grouped_map, comment_map) = group_and_sort_uses(&uses_data);
        assert_eq!(grouped_map.len(), 1, "Only one group => same prefix");
        let key = ("pub(crate)".to_string(), "std::collections".to_string());
        assert!(grouped_map.contains_key(&key));
        let finals = grouped_map[&key].clone();
        assert_eq!(finals.len(), 2, "Should have HashMap & HashSet");
        let comments = &comment_map[&key];
        assert_eq!(comments.len(), 2, "Both leading comments get appended");
    }

    // etc...
}
