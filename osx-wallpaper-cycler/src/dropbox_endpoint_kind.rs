// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_endpoint_kind.rs ]
crate::ix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropboxEndpointKind {
    OAuthToken,
    ListFolder,
    ListFolderContinue,
    Download,
}

impl std::fmt::Display for DropboxEndpointKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OAuthToken => write!(f, "dropbox.oauth2.token"),
            Self::ListFolder => write!(f, "dropbox.files.list_folder"),
            Self::ListFolderContinue => write!(f, "dropbox.files.list_folder.continue"),
            Self::Download => write!(f, "dropbox.files.download"),
        }
    }
}

#[cfg(test)]
mod dropbox_endpoint_kind_display_contract_suite {
    use super::*;

    #[traced_test]
    fn display_is_stable_and_unique_across_variants() {
        let a = DropboxEndpointKind::OAuthToken.to_string();
        let b = DropboxEndpointKind::ListFolder.to_string();
        let c = DropboxEndpointKind::ListFolderContinue.to_string();
        let d = DropboxEndpointKind::Download.to_string();

        assert_eq!(a, "dropbox.oauth2.token");
        assert_eq!(b, "dropbox.files.list_folder");
        assert_eq!(c, "dropbox.files.list_folder.continue");
        assert_eq!(d, "dropbox.files.download");

        let set: std::collections::HashSet<String> = [a, b, c, d].into_iter().collect();
        assert_eq!(set.len(), 4);
    }
}
