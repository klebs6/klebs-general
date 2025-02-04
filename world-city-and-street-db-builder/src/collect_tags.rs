// ---------------- [ File: src/collect_tags.rs ]
crate::ix!();

/// Collect tags into a [`HashMap`], generic over any iterator of
/// `(&str, &str)`.  This allows both real `TagIter<'_>` from `osmpbf`
/// **and** test mocks to be used.
///
pub fn collect_tags<'a,I>(tags: I) -> HashMap<String, String>
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    tags.map(|(k,v)| (k.to_string(), v.to_string())).collect()
}
