// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_protocol_violation_detail.rs ]
crate::ix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropboxProtocolViolationDetail {
    MissingCursor,
    MissingEntries,
    MissingPathLower,
    MissingId,
    MissingName,
    MissingAccessToken,
    InvalidJsonShape,
}

impl std::fmt::Display for DropboxProtocolViolationDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCursor      => write!(f, "missing cursor"),
            Self::MissingEntries     => write!(f, "missing entries"),
            Self::MissingPathLower   => write!(f, "missing path_lower"),
            Self::MissingId          => write!(f, "missing id"),
            Self::MissingName        => write!(f, "missing name"),
            Self::MissingAccessToken => write!(f, "missing access_token"),
            Self::InvalidJsonShape   => write!(f, "invalid json shape"),
        }
    }
}

#[cfg(test)]
mod dropbox_protocol_violation_detail_display_contract_suite {
    use super::*;

    #[traced_test]
    fn display_strings_are_non_empty_and_distinct_for_each_variant() {
        let variants = [
            DropboxProtocolViolationDetail::MissingCursor,
            DropboxProtocolViolationDetail::MissingEntries,
            DropboxProtocolViolationDetail::MissingPathLower,
            DropboxProtocolViolationDetail::MissingId,
            DropboxProtocolViolationDetail::MissingName,
            DropboxProtocolViolationDetail::MissingAccessToken,
            DropboxProtocolViolationDetail::InvalidJsonShape,
        ];

        let mut set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for v in variants.iter() {
            let s = v.to_string();
            assert!(!s.trim().is_empty());
            set.insert(s);
        }
        assert_eq!(set.len(), variants.len());
    }
}
