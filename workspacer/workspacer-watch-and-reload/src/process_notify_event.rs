// ---------------- [ File: src/process_notify_event.rs ]
crate::ix!();

// ------------------------------------------------------------------------
// Subroutine #3: Process a single notify::Result<notify::Event>
// ------------------------------------------------------------------------
pub async fn process_notify_event<P,H>(
    workspace: &Workspace<P,H>,
    event: Result<notify::Event, notify::Error>,
    tx:    Option<&mpsc::Sender<Result<(), WorkspaceError>>>,
    runner: &Arc<dyn CommandRunner + Send + Sync + 'static>,
) -> Result<(), WorkspaceError>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: WatchAndReload<Error=CrateError> + RebuildOrTest<Error=CrateError> + CrateHandleInterface<P> + Send + Sync,
{
    match event {
        Ok(ev) => {
            for path in ev.paths.iter() {
                handle_path_change(workspace, path, tx, runner).await?;
            }
        }
        Err(e) => {
            error!("File watch error: {:?}", e);
            let e: Arc<notify::Error> = Arc::new(e);
            if let Some(sender) = tx {
                let _ = sender.send(Err(WorkspaceError::from(e.clone()))).await;
            }
            return Err(WorkspaceError::from(e));
        }
    }
    Ok(())
}

#[cfg(test)]
#[disable]
mod test_process_notify_event {
    use super::*;
    use notify::EventKind;
    use std::path::PathBuf;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_process_notify_event_ok_paths() {
        let workspace = mock_workspace();
        let runner = Arc::new(MockRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        let event = notify::Event {
            kind: EventKind::Create(notify::event::CreateKind::File),
            paths: vec![PathBuf::from("Cargo.toml")],
            attrs: Default::default(),
        };
        let result = process_notify_event(&workspace, Ok(event), Some(&tx), &runner).await;
        assert!(result.is_ok());
        // Possibly read from rx if we want to see if rebuild was triggered
        let msg = rx.try_recv();
        assert!(msg.is_ok(), "Expected a rebuild result to be sent");
    }

    #[tokio::test]
    async fn test_process_notify_event_err() {
        let workspace = mock_workspace();
        let runner = Arc::new(MockRunner::default());
        let (tx, mut rx) = mpsc::channel::<Result<(), WorkspaceError>>(1);

        let fake_err = notify::Error::generic("some watch error");
        let result = process_notify_event(&workspace, Err(fake_err), Some(&tx), &runner).await;
        assert!(result.is_err(), "Should propagate watch error");
        if let Some(Err(e)) = rx.recv().await {
            println!("Got an error in the channel: {:?}", e);
        }
    }
}
