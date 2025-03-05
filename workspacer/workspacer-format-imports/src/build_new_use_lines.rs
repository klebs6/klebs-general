// ---------------- [ File: workspacer-format-imports/src/build_new_use_lines.rs ]
crate::ix!();

/**
  Updated build_new_use_lines to accept the trailing_comments map as a third parameter.
  If your existing tests only pass two arguments, just add `&BTreeMap::new()` as the third.
*/
pub fn build_new_use_lines(
    grouped_map: &BTreeMap<(String, String), BTreeSet<String>>,
    comment_map: &BTreeMap<(String, String), Vec<String>>,
    trailing_comments: &BTreeMap<(String, String, String), String>,
) -> String {
    info!("build_new_use_lines => start");
    debug!(
        "grouped_map.len()={}, comment_map.len()={}, trailing_comments.len()={}",
        grouped_map.len(),
        comment_map.len(),
        trailing_comments.len()
    );

    // If map is empty => return "\n"
    if grouped_map.is_empty() {
        debug!("grouped_map empty => returning single newline");
        info!("build_new_use_lines => done");
        return "\n".to_string();
    }

    // Convert to a vector of (vis, prefix, finals)
    let mut entries: Vec<(String, String, Vec<String>)> = Vec::new();
    for ((vis, prefix), set_of_finals) in grouped_map {
        let mut finals_vec: Vec<_> = set_of_finals.iter().cloned().collect();
        finals_vec.sort();
        entries.push((vis.clone(), prefix.clone(), finals_vec));
    }

    // Sort them by prefix, then vis (or in the order you want)
    entries.sort_by(|(va, pa, _), (vb, pb, _)|
        pa.cmp(pb).then_with(|| va.cmp(vb))
    );

    let mut new_use_lines = Vec::new();

    for (vis, prefix, finals_vec) in entries {
        // If we have leading comments in comment_map => add them now
        if let Some(cmts) = comment_map.get(&(vis.clone(), prefix.clone())) {
            for c in cmts {
                new_use_lines.push(c.clone());
            }
        }

        // Build the "use" line(s)
        if finals_vec.len() > 1 {
            // multiple finals => brace them
            let joined = finals_vec.join(", ");
            if prefix.is_empty() {
                new_use_lines.push(format!("{} use {};", vis, joined));
            } else {
                new_use_lines.push(format!("{} use {}::{{{}}};", vis, prefix, joined));
            }
        } else if finals_vec.len() == 1 {
            // single final => check trailing_comments
            let single_final = &finals_vec[0];
            // generate e.g. "pub use crate::foo::Bar;"
            let base_line = if prefix.is_empty() {
                format!("{} use {};", vis, single_final)
            } else {
                format!("{} use {}::{};", vis, prefix, single_final)
            };

            // see if there's a trailing comment
            let triple_key = (vis.clone(), prefix.clone(), single_final.clone());
            if let Some(tr_comment) = trailing_comments.get(&triple_key) {
                // e.g. => "pub use crate::foo::Bar; // trailing comment"
                // we'll put a space if comment doesn't start with "//"
                // or if it does, just put a space
                if tr_comment.starts_with("//") {
                    new_use_lines.push(format!("{} {}", base_line, tr_comment));
                } else {
                    new_use_lines.push(format!("{} //{}", base_line, tr_comment));
                }
            } else {
                // no trailing comment => push the base line
                new_use_lines.push(base_line);
            }
        } else {
            // finals_vec is empty => edge case => skip or do something else
        }
    }

    // trailing newline
    new_use_lines.push(String::new());
    let block = new_use_lines.join("\n");
    debug!(
        "build_new_use_lines => final block length={}",
        block.len()
    );
    info!("build_new_use_lines => done");
    block
}

#[cfg(test)]
mod test_build_new_use_lines {
    use super::*;

    #[traced_test]
    fn test_single_group() {
        info!("test_single_group => start");
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

        let trailing_comments = BTreeMap::new();

        let out = build_new_use_lines(&grouped_map, &comment_map, &trailing_comments);
        debug!("Got output => {:?}", out);
        assert!(out.contains("// comment1\n"));
        assert!(out.contains("// comment2\n"));
        assert!(out.contains("pub(crate) use std::collections::{HashMap, HashSet}"));
        info!("test_single_group => success");
    }

    #[traced_test]
    fn test_empty_map() {
        info!("test_empty_map => start");
        let grouped_map = BTreeMap::new();
        let comment_map = BTreeMap::new();
        // ADDED: also pass an empty BTreeMap for trailing_comments
        let trailing_comments = BTreeMap::new();

        let out = build_new_use_lines(&grouped_map, &comment_map, &trailing_comments);
        debug!("Result => {:?}", out);
        assert_eq!(out, "\n", "Should just be a single newline");
        info!("test_empty_map => success");
    }
}
