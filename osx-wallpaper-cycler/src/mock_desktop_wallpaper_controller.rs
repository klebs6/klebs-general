// ---------------- [ File: osx-wallpaper-cycler/src/mock_desktop_wallpaper_controller.rs ]
crate::ix!();

#[cfg(test)]
#[derive(Debug)]
pub struct MockDesktopWallpaperController {
    desktop_count: usize,
    applied: std::sync::Mutex<Vec<std::path::PathBuf>>,
}

#[cfg(test)]
impl MockDesktopWallpaperController {
    pub(crate) fn new(desktop_count: usize) -> Self {
        Self {
            desktop_count,
            applied: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn applied_paths(&self) -> Vec<std::path::PathBuf> {
        self.applied.lock().unwrap().clone()
    }
}

#[cfg(test)]
impl DesktopWallpaperController for MockDesktopWallpaperController {
    fn query_desktop_count(&self) -> Result<usize, WallpaperRotatorError> {
        Ok(self.desktop_count)
    }

    fn apply_wallpapers_to_desktops(
        &self,
        _scope: WallpaperRotatorScope,
        desktop_image_paths: &[std::path::PathBuf],
    ) -> Result<(), WallpaperRotatorError> {
        self.applied
            .lock()
            .unwrap()
            .extend_from_slice(desktop_image_paths);
        Ok(())
    }
}
