// ---------------- [ File: src/best_fuzzy_matches.rs ]
crate::ix!();

/// Utility: pick top `limit` fuzzy matches.
pub fn best_fuzzy_matches(
    matcher: &SkimMatcherV2,
    input: &str,
    candidates: &[String],
    limit: usize,
) -> Vec<String> {
    let mut scored = Vec::new();
    for c in candidates {
        if let Some(score) = matcher.fuzzy_match(c, input) {
            scored.push((c.clone(), score));
        }
    }
    // sort descending
    scored.sort_by_key(|(_c,score)| -(*score));
    scored.truncate(limit);
    scored.into_iter().map(|(c,_score)| c).collect()
}
