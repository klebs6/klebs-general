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

#[cfg(test)]
mod filenames_match_tests {
    use super::*;

    #[traced_test]
    fn test_exact_match_ignoring_case() {
        // The function checks eq_ignore_ascii_case first,
        // so if "MaRyLaNd-lAtEsT.osm.pbf" == "maryland-latest.osm.pbf" ignoring case => true
        let expected = "maryland-latest.osm.pbf";
        let actual = "MaRyLaNd-LaTeSt.OsM.PbF";
        assert!(filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_leading_dot_slash_stripping() {
        // If expected => "./maryland-latest.osm.pbf" 
        //   actual => "./MaRyLaNd-Latest.osm.pbf"
        // They become "maryland-latest.osm.pbf" and "MaRyLaNd-Latest.osm.pbf" => ignoring case => match
        let expected = "./maryland-latest.osm.pbf";
        let actual = "./MaRyLaNd-LaTeSt.osm.pbf";
        assert!(filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_md5_substring_case_insensitive() {
        // e.g. "maryland-latest.osm.pbf" => "maryland-latest.s0m3Md5.osm.pbf"
        // ignoring case => still match
        let expected = "maryland-latest.osm.pbf";
        let actual   = "MaRyLaNd-LaTeSt.s0M3mD5.OsM.PbF";
        assert!(filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_md5_substring_no_leading_dot() {
        // e.g. actual => "maryland-latestS0m3Md5.osm.pbf"
        // after removing .osm.pbf => we see "maryland-latestS0m3Md5"
        // does it start with "maryland-latest"? yes => remainder => "S0m3Md5"
        // but remainder does NOT start with '.' => return false
        let expected = "maryland-latest.osm.pbf";
        let actual   = "maryland-latestS0m3Md5.osm.pbf";
        assert!(!filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_exact_same_case() {
        let expected = "maryland-latest.osm.pbf";
        let actual   = "maryland-latest.osm.pbf";
        // eq_ignore_ascii_case => short-circuits => true
        assert!(filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_different_suffix() {
        // must end with .osm.pbf ignoring case or fail
        let expected = "maryland-latest.osm.pbf";
        let actual   = "maryland-latest.osm.pb"; // missing 'f'
        assert!(!filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_remainder_dot_only() {
        // If the remainder is ".", that does not qualify as an MD5. 
        // The code says remainder must start with '.' AND have >1 length => Some MD5 => ok
        // If remainder is '.' => length=1 => returns false
        let expected = "maryland-latest.osm.pbf";
        let actual   = "maryland-latest..osm.pbf"; 
        // after removing .osm.pbf => actual => "maryland-latest."
        // remainder => "." => length=1 => returns false
        assert!(!filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_empty_remainder_but_not_equal_ignore_case() {
        // e.g. "maryland-latest" != "maryland-otherstuff" ignoring case => we do a final fallback check
        // Actually the function does eq_ignore_ascii_case early if everything is identical ignoring case => returns true.
        // If they differ => we keep going => we eventually do the .osm.pbf check => the remainder is empty => but they'd differ
        let expected = "maryland-latest.osm.pbf";
        let actual   = "maryland-otherstuff.osm.pbf";
        assert!(!filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_same_base_no_md5_substring() {
        // "maryland-latest.osm.pbf" vs "MaRyLaNd-LaTeSt.osm.pbf"
        // eq_ignore_ascii_case => false, because we have a dash difference => wait, no, dashes are same length though
        // "maryland-latest" vs "maryland-latest"? Actually they match ignoring case, so eq_ignore_ascii_case is true => short-circuit => returns true
        // If we assume there's a difference in spelling => let's check the next step:
        // We'll produce a scenario that eq_ignore_ascii_case fails but the "base" is the same ignoring punctuation:
        // Actually the function doesn't skip punctuation. We'll do a direct example:
        let expected = "maryland-latest.osm.pbf";
        let actual = "marYland-latest.osm.pbf"; 
        // eq_ignore_ascii_case => same length => "maryland-latest.osm.pbf" vs "maryland-latest.osm.pbf"? Actually ignoring ASCII case => they're the same => short-circuits => true
        // let's do a scenario where the eq_ignore_ascii_case doesn't short-circuit but the base is the same ignoring case:
        
        // We'll do something that changes the prefix but ignoring case they're the same substring for the base. 
        // e.g. "maryland-latestX.osm.pbf" => remainder => "X"? => that won't match. We'll do:
        let expected2 = "maryland-latest.osm.pbf";
        let actual2   = "Maryland-latest.osm.pbf"; // Actually eq_ignore_ascii_case => true => short-circuit => returns true
        assert!(filenames_match(expected2, actual2));
    }

    #[traced_test]
    fn test_strip_leading_dot_slash() {
        // show that both have leading "./"
        let expected = "./maryland-latest.osm.pbf";
        let actual   = "./maryland-latest.osm.pbf";
        // ignoring ASCII case => short-circuit => true
        assert!(filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_completely_different_filenames() {
        // e.g. "foo-bar.osm.pbf" vs. "some-other.osm.pbf"
        // eq_ignore_ascii_case => false, so we do the suffix check => it's okay => we remove .osm.pbf => "foo-bar" vs "some-other"
        // "some-other".starts_with("foo-bar") => false => => return false
        let expected = "foo-bar.osm.pbf";
        let actual   = "some-other.osm.pbf";
        assert!(!filenames_match(expected, actual));
    }

    #[traced_test]
    fn test_md5_substring_but_wrong_suffix() {
        // "maryland-latest.osm.pbf" => "maryland-latest.abc123.osm.pb"
        // missing the 'f' => fail
        let expected = "maryland-latest.osm.pbf";
        let actual   = "maryland-latest.abc123.osm.pb";
        assert!(!filenames_match(expected, actual));
    }
}
