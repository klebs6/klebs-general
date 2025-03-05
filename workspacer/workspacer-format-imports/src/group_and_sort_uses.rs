// ---------------- [ File: workspacer-format-imports/src/group_and_sort_uses.rs ]
crate::ix!();

/**
  Now we always return three items: (grouped_map, comment_map, trailing_comments).
  If you have code/tests that only care about grouped_map & comment_map,
  just destructure as: 
       let (grouped_map, comment_map, _trailing) = group_and_sort_uses(...);
  ignoring the trailing_comments value.
*/
pub fn group_and_sort_uses(
    uses_data: &[UseItemInfo],
) -> (
    BTreeMap<(String, String), BTreeSet<String>>,
    BTreeMap<(String, String), Vec<String>>,
    BTreeMap<(String, String, String), String>, // trailing_comments
) {
    info!("group_and_sort_uses => start");
    debug!("Received uses_data with length={}", uses_data.len());

    let mut grouped_map = BTreeMap::new();
    let mut comment_map = BTreeMap::new();
    let mut trailing_comments = BTreeMap::new();

    for ud in uses_data {
        trace!(
            "Processing UseItemInfo => vis={:?}, path_list={:?}, trailing_comment={:?}",
            ud.visibility(),
            ud.path_list(),
            ud.trailing_comment()
        );

        let (prefix, finals) = split_path_into_prefix_and_final(ud.path_list());
        let key = (ud.visibility().clone(), prefix.clone());

        // 1) Insert leading comments into comment_map
        comment_map
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .extend(ud.leading_comments().clone());

        // 2) If exactly one final + trailing_comment => store that final normally plus record the trailing
        if finals.len() == 1 {
            let single_final = &finals[0];
            grouped_map
                .entry(key.clone())
                .or_insert_with(BTreeSet::new)
                .insert(single_final.clone());

            if let Some(tc) = ud.trailing_comment() {
                // store the trailing comment in trailing_comments keyed by (vis, prefix, single_final)
                let triple_key = (ud.visibility().clone(), prefix.clone(), single_final.clone());
                trailing_comments.insert(triple_key, tc.clone());
            }
        } else {
            // multiple finals or zero => no single-line trailing comment
            grouped_map
                .entry(key.clone())
                .or_insert_with(BTreeSet::new)
                .extend(finals);
        }
    }

    debug!(
        "group_and_sort_uses => grouped_map size={}, comment_map size={}, trailing_comments size={}",
        grouped_map.len(),
        comment_map.len(),
        trailing_comments.len()
    );
    info!("group_and_sort_uses => done");
    (grouped_map, comment_map, trailing_comments)
}

#[cfg(test)]
mod test_group_and_sort_uses {
    use super::*;

    /// 1) If we pass an empty list => returns empty maps.
    #[test]
    fn test_empty_list() {
        info!("test_empty_list => start");
        let uses_data = vec![];
        let (grouped_map, comment_map, trailing_comments) = group_and_sort_uses(&uses_data);
        debug!("Result => grouped_map.len()={}, comment_map.len()={}", grouped_map.len(), comment_map.len());
        assert!(grouped_map.is_empty());
        assert!(comment_map.is_empty());
        info!("test_empty_list => success");
    }

    /// 2) Two `use` items with same prefix => they merge.
    #[test]
    fn test_merge_same_prefix() {
        info!("test_merge_same_prefix => start");
        let item1 = UseItemInfoBuilder::default()
            .visibility("pub(crate)".to_string())
            .path_list("std::collections::HashMap".to_string())
            .leading_comments(vec!["// comment1\n".to_string()])
            .raw_text("".to_string())
            .trailing_comment(None)
            .range_start(0_usize)
            .range_end(0_usize)
            .build().unwrap();

        let item2 = UseItemInfoBuilder::default()
            .visibility("pub(crate)".to_string())
            .path_list("std::collections::HashSet".to_string())
            .leading_comments(vec!["// comment2\n".to_string()])
            .raw_text("".to_string())
            .trailing_comment(None)
            .range_start(0_usize)
            .range_end(0_usize)
            .build().unwrap();

        let uses_data = vec![item1, item2];
        debug!("Calling group_and_sort_uses with 2 items");
        let (grouped_map, comment_map, trailing_comments) = group_and_sort_uses(&uses_data);

        assert_eq!(grouped_map.len(), 1, "Only one group => same prefix");
        let key = ("pub(crate)".to_string(), "std::collections".to_string());
        assert!(grouped_map.contains_key(&key));
        let finals = grouped_map[&key].clone();
        assert_eq!(finals.len(), 2, "Should have HashMap & HashSet");
        let comments = &comment_map[&key];
        assert_eq!(comments.len(), 2, "Both leading comments get appended");

        info!("test_merge_same_prefix => success");
    }
}
