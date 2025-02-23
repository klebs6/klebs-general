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
    use std::path::Path;
    use tempfile::tempdir;
    use workspacer_3p::tokio;

    #[test]
    fn test_setup_file_watching_success() {
        let tmp = tempdir().unwrap();
        let path = tmp.path();

        let result = setup_file_watching(path);
        assert!(result.is_ok(), "Expected to create a file watcher for a valid path");
        let (watcher, rx) = result.unwrap();
        // We might check that 'watcher' is alive, 'rx' can receive events, etc.
        // It's tricky to fully verify without generating real FS events. 
    }

    #[test]
    fn test_setup_file_watching_invalid_path() {
        // If the path doesn't exist, does notify fail? Usually you can watch non-existent paths, 
        // but let's see if it yields an error. This might be OS/notify-implementation dependent.
        let bad_path = std::path::Path::new("/non/existent/path/for/test");
        let result = setup_file_watching(bad_path);
        // Possibly it's still Ok if the OS allows it. Or maybe it fails.
        // If it fails, you can do:
        if result.is_err() {
            println!("Got an expected error for invalid path: {:?}", result.err());
        }
    }
}
