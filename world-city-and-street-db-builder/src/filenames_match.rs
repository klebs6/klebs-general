// ---------------- [ File: src/filenames_match.rs ]
crate::ix!();

/// Returns `true` if `actual` matches the `expected` ignoring ASCII case,
/// and possibly including an optional “.<md5>” insertion before `.osm.pbf`.
/// 
/// Example:
/// - expected = "maryland-latest.osm.pbf"
/// - actual   = "MaRyLaNd-LaTeSt.1c2d3f4g.oSm.PbF"
/// => returns true
pub fn filenames_match(expected: &str, actual: &str) -> bool {
    // Because of the possibility that `expected` is "./maryland-latest.osm.pbf"
    // if someone tried something else, do a quick strip leading "./" from both.
    let expected = strip_leading_dot_slash(expected);
    let actual   = strip_leading_dot_slash(actual);

    // Quick check: if ignoring ASCII case they match exactly, done.
    if actual.eq_ignore_ascii_case(&expected) {
        return true;
    }

    // Both must end with ".osm.pbf" ignoring case
    const SUFFIX: &str = ".osm.pbf";
    // For easy checks ignoring ASCII case, let's do a lowercase version
    let expected_lc = expected.to_ascii_lowercase();
    let actual_lc   = actual.to_ascii_lowercase();
    if !expected_lc.ends_with(SUFFIX) || !actual_lc.ends_with(SUFFIX) {
        return false;
    }

    // Trim off ".osm.pbf"
    let expected_base = &expected_lc[..expected_lc.len() - SUFFIX.len()];
    let actual_base   = &actual_lc[..actual_lc.len() - SUFFIX.len()];

    // actual might be something like "maryland-latest.<md5>"
    // expected might be "maryland-latest"
    if !actual_base.starts_with(expected_base) {
        return false;
    }

    // The remainder after "maryland-latest"
    let remainder = &actual_base[expected_base.len()..];
    // If remainder is empty, we already did the eq_ignore_ascii_case() check above,
    // so presumably that would have matched. If remainder starts with '.' and more stuff,
    // that's presumably the MD5. So let's check that:
    if remainder.starts_with('.') && remainder.len() > 1 {
        // e.g. ".abc1234"
        true
    } else {
        false
    }
}
