// ---------------- [ File: osx-wallpaper-cycler/src/build_wallpaper_cache_state.rs ]
crate::ix!();

pub fn build_wallpaper_cache_state(assignments: Vec<WallpaperCacheAssignment>) -> WallpaperCacheState {
    WallpaperCacheStateBuilder::default()
        .schema_version(1u32)
        .assignments(assignments)
        .build()
        .unwrap()
}
