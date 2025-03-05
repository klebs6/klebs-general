// ---------------- [ File: src/find_single_prefix_match.rs ]
crate::ix!();

/// A small helper that checks if `crate_name` starts with exactly one `prefix + "-"` from the 
/// scanned groups. If we find exactly one match, returns that prefix. If zero or multiple, `None`.
pub fn find_single_prefix_match<P,H>(crate_name: &str, groups: &[PrefixGroup<P,H>]) -> Option<String> 
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Debug + Send + Sync + Clone,
{
    let matching: Vec<_> = groups
        .iter()
        .filter(|g| crate_name.starts_with(&format!("{}-", g.prefix())))
        .collect();

    match matching.len() {
        1 => Some(matching[0].prefix().to_string()),
        0 => {
            debug!("No prefix group matches '{}'", crate_name);
            None
        }
        _ => {
            warn!("Multiple prefix groups could match '{}'; skipping auto registration", crate_name);
            None
        }
    }
}
