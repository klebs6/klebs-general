crate::ix!();

// -------------------------------------------------------------------------
// 4) Build new use lines from the grouped map
// -------------------------------------------------------------------------
pub fn build_new_use_lines(
    grouped_map: &BTreeMap<(String, String), BTreeSet<String>>,
    comment_map: &BTreeMap<(String,String), Vec<String>>
) -> String {
    let mut new_use_lines = Vec::new();

    for ((vis, prefix), finals) in grouped_map.iter() {
        let mut finals_vec: Vec<_> = finals.iter().cloned().collect();
        finals_vec.sort();
        let joined = finals_vec.join(", ");

        let line = if prefix.is_empty() {
            format!("{} use {};", vis, joined)
        } else {
            format!("{} use {}::{{{}}};", vis, prefix, joined)
        };

        if let Some(cmts) = comment_map.get(&(vis.clone(), prefix.clone())) {
            for cmt in cmts {
                new_use_lines.push(cmt.clone());
            }
        }
        new_use_lines.push(line);
    }

    // add a trailing newline
    let block = new_use_lines.join("\n") + "\n";
    block
}

#[cfg(test)]
mod test_build_new_use_lines {
    use super::build_new_use_lines;
    use std::collections::{BTreeMap, BTreeSet};

    /// 1) If grouped_map is empty => returns ""
    #[test]
    fn test_empty_map() {
        let grouped_map = BTreeMap::new();
        let comment_map = BTreeMap::new();
        let out = build_new_use_lines(&grouped_map, &comment_map);
        assert_eq!(out, "\n", "Should just be a single newline");
    }

    /// 2) Single group => ensures we place leading comments, then one line
    #[test]
    fn test_single_group() {
        let mut grouped_map = BTreeMap::new();
        let mut comment_map = BTreeMap::new();
        let key = ("pub(crate)".to_string(), "std::collections".to_string());
        let mut set = BTreeSet::new();
        set.insert("HashMap".to_string());
        set.insert("HashSet".to_string());
        grouped_map.insert(key.clone(), set);

        comment_map.insert(
            key.clone(),
            vec!["// comment1\n".to_string(), "// comment2\n".to_string()],
        );

        let out = build_new_use_lines(&grouped_map, &comment_map);
        // expect something like:
        //   // comment1
        //   // comment2
        //   pub(crate) use std::collections::{HashMap, HashSet};
        //   \n
        // We'll do basic checks
        assert!(out.contains("// comment1\n"));
        assert!(out.contains("// comment2\n"));
        assert!(out.contains("pub(crate) use std::collections::{HashMap, HashSet}"));
    }

    // etc...
}
