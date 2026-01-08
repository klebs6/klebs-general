// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_access_token_state.rs ]
crate::ix!();

#[derive(Debug, Clone, getset::Getters)]
#[getset(get = "pub(crate)")]
pub struct DropboxAccessTokenState {
    token: String,
    expires_at: std::time::Instant,
}

impl DropboxAccessTokenState {
    pub(crate) fn new(token: String, expires_at: std::time::Instant) -> Self {
        Self { token, expires_at }
    }
}

#[cfg(test)]
mod dropbox_access_token_state_contract_suite {
    use super::*;

    #[traced_test]
    fn new_sets_fields_and_getters_return_expected_values() {
        let expires_at = std::time::Instant::now() + std::time::Duration::from_secs(123);
        let state = DropboxAccessTokenState::new("tok".to_string(), expires_at);

        assert_eq!(state.token(), "tok");
        assert_eq!(*state.expires_at(), expires_at);
    }

    #[traced_test]
    fn clone_preserves_token_and_expiration() {
        let expires_at = std::time::Instant::now() + std::time::Duration::from_secs(7);
        let a = DropboxAccessTokenState::new("tok2".to_string(), expires_at);
        let b = a.clone();

        assert_eq!(b.token(), "tok2");
        assert_eq!(*b.expires_at(), expires_at);
    }
}
