// ---------------- [ File: src/setup_file_watching.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #1: Setup the file watcher
// ------------------------------------------------------------------------
pub fn setup_file_watching(
    workspace_path: &Path,
) -> Result<(RecommendedWatcher, async_channel::Receiver<notify::Result<notify::Event>>), WatchError>
{
    // We'll use an async channel to push notify events to our async loop
    let (notify_tx, notify_rx) = async_channel::unbounded();

    // Create the `RecommendedWatcher`
    let notify_tx_clone = notify_tx.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            // This closure is invoked on every filesystem event from notify
            let _ = notify_tx_clone.try_send(res);
        },
        notify::Config::default(),
    )
    .map_err(|e| WatchError::NotifyError(e.into()))?;

    // Start watching
    watcher
        .watch(workspace_path, RecursiveMode::Recursive)
        .map_err(|e| WatchError::NotifyError(e.into()))?;

    Ok((watcher, notify_rx))
}

#[cfg(test)]
mod test_setup_file_watching {
    use super::*;

    // Already enabled, but we switch to traced_test and add logging:

    #[traced_test]
    fn test_setup_file_watching_success() {
        info!("Starting test_setup_file_watching_success");
        let tmp = tempdir().unwrap();
        let path = tmp.path();

        let result = setup_file_watching(path);
        assert!(result.is_ok(), "Expected to create a file watcher for a valid path");
        let (_watcher, _rx) = result.unwrap();
    }

    #[traced_test]
    fn test_setup_file_watching_invalid_path() {
        info!("Starting test_setup_file_watching_invalid_path");
        let bad_path = Path::new("/non/existent/path/for/test");
        let result = setup_file_watching(bad_path);
        if result.is_err() {
            warn!("Got an error for invalid path, which is expected on some platforms: {:?}", result.err());
        }
    }
}
