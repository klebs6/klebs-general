// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_wallpaper_candidate.rs ]
crate::ix!();

#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    getset::Getters,
    derive_builder::Builder
)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct DropboxWallpaperCandidate {
    id: String,
    name: String,
    path_lower: String,
}

impl DropboxWallpaperCandidate {
    pub fn file_extension_lowercase(&self) -> Option<String> {
        let name = self.name.as_str();
        let (_, ext) = name.rsplit_once('.')?;
        let normalized = ext.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        }
    }
}

#[cfg(test)]
mod dropbox_wallpaper_candidate_extension_contract_suite {
    use super::*;

    fn build_candidate_with_name(name: &str) -> DropboxWallpaperCandidate {
        DropboxWallpaperCandidateBuilder::default()
            .id("id:test")
            .name(name)
            .path_lower("/wallpapers/test")
            .build()
            .unwrap()
    }

    #[traced_test]
    fn builder_sets_fields_and_getters_return_expected_values() {
        let c = DropboxWallpaperCandidateBuilder::default()
            .id("id:abc")
            .name("pic.JPG")
            .path_lower("/wallpapers/pic.JPG")
            .build()
            .unwrap();

        assert_eq!(c.id(), "id:abc");
        assert_eq!(c.name(), "pic.JPG");
        assert_eq!(c.path_lower(), "/wallpapers/pic.JPG");
    }

    #[traced_test]
    fn file_extension_lowercase_is_none_when_no_dot_or_empty_extension() {
        let a = build_candidate_with_name("noext");
        assert_eq!(a.file_extension_lowercase(), None);

        let b = build_candidate_with_name("trailing.");
        assert_eq!(b.file_extension_lowercase(), None);

        let c = build_candidate_with_name("also.trailing.   ");
        assert_eq!(c.file_extension_lowercase(), None);
    }

    #[traced_test]
    fn file_extension_lowercase_normalizes_case_and_trims_whitespace() {
        let a = build_candidate_with_name("photo.JPG");
        assert_eq!(a.file_extension_lowercase().as_deref(), Some("jpg"));

        let b = build_candidate_with_name("photo.  HeIc  ");
        assert_eq!(b.file_extension_lowercase().as_deref(), Some("heic"));

        let c = build_candidate_with_name("photo.  tIfF\t");
        assert_eq!(c.file_extension_lowercase().as_deref(), Some("tiff"));
    }

    #[traced_test]
    fn file_extension_lowercase_uses_last_dot_and_handles_leading_dot_files() {
        let a = build_candidate_with_name("a.b.c.PNG");
        assert_eq!(a.file_extension_lowercase().as_deref(), Some("png"));

        let b = build_candidate_with_name(".bashrc");
        assert_eq!(b.file_extension_lowercase().as_deref(), Some("bashrc"));

        let c = build_candidate_with_name("..double.dot.JPG");
        assert_eq!(c.file_extension_lowercase().as_deref(), Some("jpg"));
    }
}
