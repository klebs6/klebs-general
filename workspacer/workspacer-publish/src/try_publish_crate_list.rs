crate::ix!();

#[async_trait]
pub trait TryPublishCrateList<P, H> {
    async fn try_publish_crate_list(
        &self,
        root_crate_names: &[String],
        dry_run: bool,
    ) -> Result<(), WorkspaceError>;
}

#[async_trait]
impl<P, H> TryPublishCrateList<P, H> for Workspace<P, H>
where
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,
    H: TryPublish<Error = CrateError>
        + CrateHandleInterface<P>
        + IsPrivate<Error = CrateError>
        + Send
        + Sync
        + 'static,
{
    async fn try_publish_crate_list(
        &self,
        root_crate_names: &[String],
        dry_run: bool,
    ) -> Result<(), WorkspaceError> {
        use tracing::{debug, error, info, trace};

        trace!(
            "Entered TryPublishCrateList::try_publish_crate_list with {} requested crate(s)",
            root_crate_names.len()
        );

        if root_crate_names.is_empty() {
            info!("No crates provided to try_publish_crate_list; nothing to publish.");
            return Ok(());
        }

        let mut unique_roots = std::collections::BTreeSet::<String>::new();
        for name in root_crate_names {
            if !unique_roots.insert(name.clone()) {
                debug!(
                    "Duplicate crate '{}' found in requested publish list; ignoring duplicate entry.",
                    name
                );
            }
        }

        info!(
            "Prepared crate-list publish for {} unique root crate(s): {:?}",
            unique_roots.len(),
            unique_roots
        );

        for root in unique_roots.iter() {
            info!(
                "Publishing crate-tree rooted at '{}' as part of crate-list publish.",
                root
            );
            if let Err(err) = self.try_publish_crate_tree(root, dry_run).await {
                error!(
                    "Error while publishing crate-tree rooted at '{}' in crate-list publish: {:?}",
                    root,
                    err
                );
                return Err(err);
            }
        }

        info!("Completed crate-list publish for all requested crate roots.");
        Ok(())
    }
}
