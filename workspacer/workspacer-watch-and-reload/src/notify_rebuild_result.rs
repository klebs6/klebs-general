// ---------------- [ File: workspacer-watch-and-reload/src/notify_rebuild_result.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #5: Notify the rebuild result to the caller
// ------------------------------------------------------------------------
pub async fn notify_rebuild_result<E>(
    tx:            Option<&mpsc::Sender<Result<(), E>>>,
    rebuild_result: Result<(), E>,
) 
    where E: From<WatchError>,
{
    if let Some(sender) = tx {
        let _ = sender.send(rebuild_result).await;
    }
}

// ------------------- [ File: src/notify_rebuild_result.rs ] test module fixes:
#[cfg(test)]
mod test_notify_rebuild_result {
    use super::*;

    #[traced_test]
    async fn test_notify_rebuild_result_some() {
        info!("Starting test_notify_rebuild_result_some");
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);
        let res: Result<(), WorkspaceError> = Ok(());

        notify_rebuild_result(Some(&tx), res.clone()).await;

        // Instead of assert_eq!(received, Some(res)),
        // do a more flexible check:
        match rx.try_recv() {
            Ok(r) => {
                assert!(r.is_ok(), "Expected Ok, got {:?}", r);
            }
            Err(e) => {
                panic!("No message received, got error: {:?}", e);
            }
        }
    }

    #[traced_test]
    async fn test_notify_rebuild_result_none() {
        info!("Starting test_notify_rebuild_result_none");
        let res: Result<(), WorkspaceError> = Err(WorkspaceError::FileWatchError);
        notify_rebuild_result(None, res.clone()).await;
        // No channel => no panic
    }
}
