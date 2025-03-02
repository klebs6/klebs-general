// ---------------- [ File: src/build_new_use_lines.rs ]
crate::ix!();

pub fn build_new_use_lines(
    grouped_map: &BTreeMap<(String, String), BTreeSet<String>>,
    comment_map: &BTreeMap<(String,String), Vec<String>>
) -> String {
    info!("build_new_use_lines => start");
    debug!(
        "grouped_map.len()={}, comment_map.len()={}",
        grouped_map.len(),
        comment_map.len()
    );

    // If everything is empty, return exactly "\n" to satisfy test_empty_map
    if grouped_map.is_empty() {
        debug!("grouped_map empty => returning single newline");
        info!("build_new_use_lines => done");
        return "\n".to_string();
    }

    let mut new_use_lines = Vec::new();

    for ((vis, prefix), finals) in grouped_map.iter() {
        trace!(
            "Processing group => vis={:?}, prefix={:?}, finals.len()={}",
            vis,
            prefix,
            finals.len()
        );

        let mut finals_vec: Vec<_> = finals.iter().cloned().collect();
        finals_vec.sort();

        // If there's only one final, "prefix::final" not "prefix::{final}" 
        let line = if finals_vec.len() == 1 {
            let single = &finals_vec[0];
            if prefix.is_empty() {
                // e.g. "pub use X;" or just "use X;"
                format!("{} use {};", vis, single)
            } else {
                format!("{} use {}::{};", vis, prefix, single)
            }
        } else {
            // multiple finals => brace them
            let joined = finals_vec.join(", ");
            if prefix.is_empty() {
                format!("{} use {};", vis, joined)
            } else {
                format!("{} use {}::{{{}}};", vis, prefix, joined)
            }
        };

        // Insert any leading comments for this group
        if let Some(cmts) = comment_map.get(&(vis.clone(), prefix.clone())) {
            debug!(
                "Found {} comment lines for group (vis={}, prefix={}); appending them",
                cmts.len(),
                vis,
                prefix
            );
            for cmt in cmts {
                new_use_lines.push(cmt.clone());
            }
        }

        new_use_lines.push(line);
    }

    // Add a trailing newline
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
    fn test_empty_map() {
        info!("test_empty_map => start");
        let grouped_map = BTreeMap::new();
        let comment_map = BTreeMap::new();
        let out = build_new_use_lines(&grouped_map, &comment_map);
        debug!("Result => {:?}", out);
        assert_eq!(out, "\n", "Should just be a single newline");
        info!("test_empty_map => success");
    }

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

        let out = build_new_use_lines(&grouped_map, &comment_map);
        debug!("Got output => {:?}", out);
        assert!(out.contains("// comment1\n"));
        assert!(out.contains("// comment2\n"));
        assert!(out.contains("pub(crate) use std::collections::{HashMap, HashSet}"));
        info!("test_single_group => success");
    }
}
