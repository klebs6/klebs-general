// ---------------- [ File: workspacer-cli/src/bump_release_arg.rs ]
crate::ix!();

/// A helper struct for specifying how to bump version (major, minor, patch, alphaN).
/// Example CLI usage: `--release patch` or `--release alpha=3`
#[derive(Debug,Clone)]
pub struct ReleaseArg(pub ReleaseType);

impl FromStr for ReleaseArg {
    type Err = String;

    /// Parses something like "major", "minor", "patch", or "alpha[=N]"
    /// Example valid inputs:
    ///   - "major"
    ///   - "minor"
    ///   - "patch"
    ///   - "alpha"
    ///   - "alpha=1", "alpha=7", ...
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        if lower == "major" {
            Ok(ReleaseArg(ReleaseType::Major))
        } else if lower == "minor" {
            Ok(ReleaseArg(ReleaseType::Minor))
        } else if lower == "patch" {
            Ok(ReleaseArg(ReleaseType::Patch))
        } else if lower.starts_with("alpha") {
            // might be just "alpha" or "alpha=NUM"
            let parts: Vec<_> = lower.splitn(2, '=').collect();
            if parts.len() == 1 {
                // just "alpha"
                Ok(ReleaseArg(ReleaseType::Alpha(None)))
            } else {
                // alpha=NUM
                let maybe_num = parts[1].parse::<u64>().map_err(|e| format!("Invalid number in alpha=NN: {e}"))?;
                Ok(ReleaseArg(ReleaseType::Alpha(Some(maybe_num))))
            }
        } else {
            Err(format!("Unsupported release type: '{s}' (valid: major|minor|patch|alpha[=N])"))
        }
    }
}
