// ---------------- [ File: src/notify_rebuild_result.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #5: Notify the rebuild result to the caller
// ------------------------------------------------------------------------
pub async fn notify_rebuild_result(
    tx:            Option<&mpsc::Sender<Result<(), WorkspaceError>>>,
    rebuild_result: Result<(), WorkspaceError>,
) {
    if let Some(sender) = tx {
        let _ = sender.send(rebuild_result).await;
    }
}

#[cfg(test)]
mod test_notify_rebuild_result {
    use super::*;
    use tokio::sync::mpsc;

    // Re-enable by removing #[disable].
    // Switch to traced_test, add logging.

    #[traced_test]
    async fn test_notify_rebuild_result_some() {
        info!("Starting test_notify_rebuild_result_some");
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);
        let res: Result<(), WorkspaceError> = Ok(());
        notify_rebuild_result(Some(&tx), res.clone()).await;
        let received = rx.try_recv().ok();
        assert_eq!(received, Some(res));
    }

    #[traced_test]
    async fn test_notify_rebuild_result_none() {
        info!("Starting test_notify_rebuild_result_none");
        let res: Result<(), WorkspaceError> = Err(WorkspaceError::FileWatchError);
        notify_rebuild_result(None, res.clone()).await;
        // no channel => no panic
    }
}
