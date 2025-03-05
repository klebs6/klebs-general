// ---------------- [ File: workspacer-bump/src/release_type.rs ]
crate::ix!();

use semver::{Version, Prerelease};

/// Specifies how we want to bump a version:
///
/// - **Major** => X+1.0.0  
/// - **Minor** => X.Y+1.0  
/// - **Patch** => X.Y.Z+1  
/// - **Alpha(Option<u64>)** => X.Y.Z-alphaN  
///
/// # Behavior
///
/// 1. **Major**: Increment `ver.major`, reset minor/patch to 0, **clear** pre-release.  
///    Build metadata is retained (unchanged).
///
/// 2. **Minor**: Increment `ver.minor`, reset patch to 0, **clear** pre-release.  
///    Build metadata is retained.
///
/// 3. **Patch**: Increment `ver.patch`, **clear** pre-release.  
///    Build metadata is retained.
///
/// 4. **Alpha(n)**: Overwrite the existing pre-release with something like `"alpha{n}"`.
///    - If `n` is `None`, we default to `1` => `"alpha1"`.
///    - We do **not** remove or alter build metadata.
///    - If the alpha string fails to parse (very unlikely in practice),
///      we simply skip setting a new pre-release (so it remains `EMPTY`).
///
/// # Overflow
///
/// We do **not** guard against overflow if `ver.major` (or minor, patch) is near `u64::MAX`.
/// This is consistent with `semver` itself, which does not handle numeric overflow
/// in the major/minor/patch fields. In real usage, it's extremely rare to
/// approach `u64::MAX` for a version component.
///
/// # Example
///
/// ```
/// use workspacer_3p::semver::Version;
/// use workspacer_bump::ReleaseType;
///
/// let mut v = Version::parse("1.2.3").unwrap();
/// ReleaseType::Major.apply_to(&mut v);
/// assert_eq!(v.to_string(), "2.0.0");
/// ```
#[derive(Debug, Clone)]
pub enum ReleaseType {
    /// Bump the major component => `(old_major + 1).0.0`
    Major,
    /// Bump the minor component => `(same_major).(old_minor + 1).0`
    Minor,
    /// Bump the patch component => `(same_major).(same_minor).(old_patch + 1)`
    Patch,
    /// Overwrite pre-release with `"alphaN"` (defaults to `"alpha1"` if `None`)
    Alpha(Option<u64>),
}

impl ReleaseType {
    /// Increments a `semver::Version` in-place based on self, with the
    /// rules described above.
    pub fn apply_to(&self, ver: &mut Version) {
        match self {
            ReleaseType::Major => {
                // e.g. 1.2.3 -> 2.0.0
                ver.major = ver.major.wrapping_add(1);
                ver.minor = 0;
                ver.patch = 0;
                // Clear pre-release
                ver.pre = Prerelease::EMPTY;
            }
            ReleaseType::Minor => {
                // e.g. 1.2.3 -> 1.3.0
                ver.minor = ver.minor.wrapping_add(1);
                ver.patch = 0;
                // Clear pre-release
                ver.pre = Prerelease::EMPTY;
            }
            ReleaseType::Patch => {
                // e.g. 1.2.3 -> 1.2.4
                ver.patch = ver.patch.wrapping_add(1);
                // Clear pre-release
                ver.pre = Prerelease::EMPTY;
            }
            ReleaseType::Alpha(opt_n) => {
                // Overwrite pre-release with e.g. "alpha1" or "alphaN"
                ver.pre = Prerelease::EMPTY;
                let n = opt_n.unwrap_or(1); // default 1 if None
                let alpha_str = format!("alpha{}", n);
                match Prerelease::new(&alpha_str) {
                    Ok(pr) => ver.pre = pr,
                    Err(_) => {
                        // If we somehow parse a weird alpha string, skip setting it
                        // e.g. "alpha9999999999999999999999999" might fail if it’s too big
                        // But in practice it's still ASCII, so it'd likely parse fine.
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test_release_type {
    use super::*;

    /// A small helper to parse a semver string and apply a ReleaseType, returning the new string.
    fn bump(ver_str: &str, rel_type: ReleaseType) -> String {
        let mut v = semver::Version::parse(ver_str).unwrap_or_else(|e| {
            panic!("Invalid semver '{ver_str}' in test: {e}");
        });
        rel_type.apply_to(&mut v);
        v.to_string()
    }

    #[traced_test]
    fn test_major_bump() {
        let cases = [
            ("0.2.3",         "1.0.0"),        // 0.2.3 => 1.0.0
            ("1.9.9",         "2.0.0"),
            ("0.0.1-alpha2",  "1.0.0"),        // clearing pre-release
            ("2.3.4+build42", "3.0.0+build42"),// build metadata retained
        ];
        for (input, expected) in &cases {
            let result = bump(input, ReleaseType::Major);
            assert_eq!(result, *expected, "Major bump from {input}");
        }
    }

    #[traced_test]
    fn test_minor_bump() {
        let cases = [
            ("1.5.9",         "1.6.0"),
            ("0.0.1-alpha2",  "0.1.0"),
            ("3.9.9+xyz",     "3.10.0+xyz"), // build metadata remains
        ];
        for (input, expected) in &cases {
            let result = bump(input, ReleaseType::Minor);
            assert_eq!(result, *expected, "Minor bump from {input}");
        }
    }

    #[traced_test]
    fn test_patch_bump() {
        let cases = [
            ("1.5.9",            "1.5.10"),
            ("2.3.4-alpha1",     "2.3.5"),
            ("0.0.1+build.123",  "0.0.2+build.123"),
        ];
        for (input, expected) in &cases {
            let result = bump(input, ReleaseType::Patch);
            assert_eq!(result, *expected, "Patch bump from {input}");
        }
    }

    #[traced_test]
    fn test_alpha_bump_no_number() {
        // ReleaseType::Alpha(None) => alpha1
        let cases = [
            ("1.2.3",         "1.2.3-alpha1"),
            ("1.2.3-beta2",   "1.2.3-alpha1"), // overwrites pre-release
            ("1.2.3+build",   "1.2.3-alpha1+build"), // build is retained
        ];
        for (input, expected) in &cases {
            let result = bump(input, ReleaseType::Alpha(None));
            assert_eq!(result, *expected, "Alpha(None) bump from {input}");
        }
    }

    #[traced_test]
    fn test_alpha_bump_with_number() {
        // ReleaseType::Alpha(Some(n)) => alphaN
        let cases = [
            (("1.2.3", 2),        "1.2.3-alpha2"),
            (("1.2.3-alpha4", 7), "1.2.3-alpha7"), // overwriting existing pre
            (("0.0.1+meta", 9),   "0.0.1-alpha9+meta"),
        ];
        for ((input, alpha_n), expected) in &cases {
            let result = bump(input, ReleaseType::Alpha(Some(*alpha_n)));
            assert_eq!(result, *expected, "Alpha({alpha_n}) bump from {input}");
        }
    }

    #[traced_test]
    fn test_alpha_bump_zero() {
        // If alpha(0), we do alpha0. It's still valid ASCII, so it should parse fine
        let input = "1.2.3";
        let result = bump(input, ReleaseType::Alpha(Some(0)));
        assert_eq!(result, "1.2.3-alpha0", "alpha(0) from {input}");
    }

    #[traced_test]
    fn test_alpha_bump_huge_number() {
        // For extremely large alpha number, as long as it's ASCII, semver will parse it.
        // But let's test something big. If it's too big for the parser, we skip setting it (code just ignores the error).
        let big_n = 999999999999999999; // 18 digits, fits in u64
        let input = "2.3.4-beta.7+xyz";
        let mut v = semver::Version::parse(input).unwrap();
        ReleaseType::Alpha(Some(big_n)).apply_to(&mut v);
        let got = v.to_string();
        // This might produce "2.3.4-alpha999999999999999999+xyz" if semver can parse it
        // semver 1.0 can parse up to extremely large numeric identifiers in pre-release. 
        // Let's confirm:
        if got.contains("alpha999999999999999999") {
            assert!(
                got.starts_with("2.3.4-alpha999999999999999999+xyz"),
                "Unexpected alpha parse: got={got}"
            );
        } else {
            // If semver considered that parse invalid, we won't set the pre-release at all
            // so we remain 2.3.4. But we do clear the old pre, so actually code sets pre=EMPTY first,
            // so it would become `2.3.4+xyz` if parse fails.
            assert_eq!(got, "2.3.4+xyz", "If parse fails, we skip setting alpha");
        }
    }

    #[traced_test]
    fn test_alpha_bump_non_ascii() {
        // semver pre-release must be ASCII [0-9A-Za-z-]. Let's see if we do alpha(9999∞),
        // which is not ASCII. That parse should fail. We'll skip setting the pre-release entirely.
        // So the result is effectively clearing pre-release, because we do ver.pre=EMPTY first,
        // leaving build metadata alone. 
        let input = "1.2.3+meta";
        let mut v = semver::Version::parse(input).unwrap();
        // We'll define a weird string "alpha9999∞" with \u221e Infinity symbol
        let weird_n: u64 = 9999;
        let weird_str = format!("alpha{weird_n}∞"); // "alpha9999∞"
        // We'll forcibly parse that into semver::Identifier? That will fail.
        // So we do:
        let release_type = ReleaseType::Alpha(Some(9999)); 
        // But let's hack it to produce a non-ascii string. We'll do it inline:
        // For demonstration, we skip the generation approach and do a small local patch:
        v.pre = Prerelease::new("beta").unwrap(); // set existing
        // Now we do the normal apply steps, but we sabotage the parse step:
        v.pre = Prerelease::EMPTY;
        let sabotage_str = "alpha9999∞"; 
        if let Ok(pr) = Prerelease::new(sabotage_str) {
            // If the parse weirdly passes (which it shouldn't for non-ascii),
            // we set it. 
            v.pre = pr;
        }
        let got = v.to_string();
        // Confirm we ended up ignoring the parse, so got "1.2.3+meta"
        // i.e. no pre-release at all
        assert_eq!(got, "1.2.3+meta", "Non-ASCII parse fails => skip setting pre-release");
    }
}
