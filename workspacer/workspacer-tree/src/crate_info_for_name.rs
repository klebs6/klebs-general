// ---------------- [ File: workspacer-tree/src/crate_info_for_name.rs ]
crate::ix!();

/// Utility: find the version/path for the given crate_name in our crate_info list.
pub fn crate_info_for_name(
    crate_info: &[(String, Option<SemverVersion>, PathBuf, Vec<String>)],
    name: &str,
) -> (Option<SemverVersion>, PathBuf) 
{
    for (cn, cv, p, _) in crate_info {
        if cn == name {
            return (cv.clone(), p.clone());
        }
    }
    // fallback if not found
    (None, PathBuf::new())
}
