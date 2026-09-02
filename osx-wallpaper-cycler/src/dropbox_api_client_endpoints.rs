// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_api_client_endpoints.rs ]
crate::ix!();

#[derive(Debug, Clone, derive_builder::Builder)]
#[builder(setter(into))]
pub struct DropboxApiClientEndpoints {
    api_base_url: String,
    content_base_url: String,
    oauth_base_url: String,
}

impl DropboxApiClientEndpoints {
    pub fn default_dropbox() -> Self {
        Self {
            api_base_url: "https://api.dropboxapi.com".to_string(),
            content_base_url: "https://content.dropboxapi.com".to_string(),
            oauth_base_url: "https://api.dropbox.com".to_string(),
        }
    }

    pub fn api_base_url(&self) -> &str {
        self.api_base_url.as_str()
    }

    pub fn content_base_url(&self) -> &str {
        self.content_base_url.as_str()
    }

    pub fn oauth_base_url(&self) -> &str {
        self.oauth_base_url.as_str()
    }
}

#[cfg(test)]
mod dropbox_api_client_endpoints_contract_suite {
    use super::*;

    #[traced_test]
    fn default_dropbox_endpoints_match_expected_urls() {
        let e = DropboxApiClientEndpoints::default_dropbox();
        assert_eq!(e.api_base_url(), "https://api.dropboxapi.com");
        assert_eq!(e.content_base_url(), "https://content.dropboxapi.com");
        assert_eq!(e.oauth_base_url(), "https://api.dropbox.com");
    }

    #[traced_test]
    fn endpoints_builder_can_override_all_urls_independently() {
        let e = DropboxApiClientEndpointsBuilder::default()
            .api_base_url("http://api.example.test")
            .content_base_url("http://content.example.test")
            .oauth_base_url("http://oauth.example.test")
            .build()
            .unwrap();

        assert_eq!(e.api_base_url(), "http://api.example.test");
        assert_eq!(e.content_base_url(), "http://content.example.test");
        assert_eq!(e.oauth_base_url(), "http://oauth.example.test");
    }
}
