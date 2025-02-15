crate::ix!();

// A quick helper for mocking an iterator over (&str, &str).
pub fn mock_tag_iter<'a>(pairs: Vec<(&'a str, &'a str)>) -> impl Iterator<Item = (&'a str, &'a str)> {
    pairs.into_iter()
}
