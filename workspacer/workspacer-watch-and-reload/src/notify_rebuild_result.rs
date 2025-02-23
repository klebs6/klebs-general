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
#[disable]
mod test_notify_rebuild_result {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_notify_rebuild_result_some() {
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);
        let res: Result<(), WorkspaceError> = Ok(());

        notify_rebuild_result(Some(&tx), res.clone()).await;

        // check the channel
        let received = rx.try_recv();
        assert_eq!(received.ok(), Some(res));
    }

    #[tokio::test]
    async fn test_notify_rebuild_result_none() {
        let res: Result<(), WorkspaceError> = Err(WorkspaceError::FileWatchError);
        // If tx is None => does nothing
        notify_rebuild_result(None, res.clone()).await;
        // No test to do, just confirm it doesn't panic
    }
}
