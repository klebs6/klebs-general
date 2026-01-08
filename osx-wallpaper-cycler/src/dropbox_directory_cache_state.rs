// ---------------- [ File: osx-wallpaper-cycler/src/dropbox_directory_cache_state.rs ]
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
pub struct DropboxDirectoryCacheState {
    schema_version: u32,
    roots: Vec<String>,
    entries: Vec<DropboxWallpaperCandidate>,
    cached_at_epoch_seconds: u64,
}

impl DropboxDirectoryCacheState {
    pub fn cache_file_path(cache_dir: &std::path::Path) -> std::path::PathBuf {
        cache_dir.join("dropbox_directory_cache.json")
    }

    pub fn is_compatible_with_roots(&self, roots: &[String]) -> bool {
        self.roots.as_slice() == roots
    }
}

#[cfg(test)]
mod dropbox_directory_cache_state_contract_suite {
    use super::*;

    fn build_candidate(id: &str) -> DropboxWallpaperCandidate {
        DropboxWallpaperCandidateBuilder::default()
            .id(id)
            .name(format!("{id}.jpg"))
            .path_lower(format!("/wallpapers/{id}.jpg"))
            .build()
            .unwrap()
    }

    #[traced_test]
    fn cache_file_path_is_under_cache_dir() {
        let dir = std::path::PathBuf::from("/tmp/cache");
        let p = DropboxDirectoryCacheState::cache_file_path(&dir);
        assert_eq!(p, dir.join("dropbox_directory_cache.json"));
    }

    #[traced_test]
    fn builder_sets_fields_and_getters_return_expected_values() {
        let state = DropboxDirectoryCacheStateBuilder::default()
            .schema_version(1u32)
            .roots(vec!["/Wallpapers".to_string()])
            .entries(vec![build_candidate("id:1"), build_candidate("id:2")])
            .cached_at_epoch_seconds(123u64)
            .build()
            .unwrap();

        assert_eq!(*state.schema_version(), 1u32);
        assert_eq!(state.roots().as_slice(), &["/Wallpapers".to_string()]);
        assert_eq!(state.entries().len(), 2);
        assert_eq!(*state.cached_at_epoch_seconds(), 123u64);
    }

    #[traced_test]
    fn compatibility_checks_roots_in_order() {
        let state = DropboxDirectoryCacheStateBuilder::default()
            .schema_version(1u32)
            .roots(vec!["/A".to_string(), "/B".to_string()])
            .entries(vec![build_candidate("id:1")])
            .cached_at_epoch_seconds(1u64)
            .build()
            .unwrap();

        assert!(state.is_compatible_with_roots(&vec!["/A".to_string(), "/B".to_string()]));
        assert!(!state.is_compatible_with_roots(&vec!["/B".to_string(), "/A".to_string()]));
        assert!(!state.is_compatible_with_roots(&vec!["/A".to_string()]));
    }
}
