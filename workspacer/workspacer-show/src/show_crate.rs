crate::ix!();

/// Subroutine for handling `ws show crate` subcommand: single crate only.
/// We do *not* attempt to create a workspace. We simply confirm that the given path is a single crate.
#[tracing::instrument(level = "trace", skip(flags))]
pub async fn show_crate(flags: &ShowFlags) -> Result<String, WorkspaceError> {

    trace!("User chose subcommand: ws show crate");

    let crate_path = flags
        .path()
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    debug!("crate path: {:?}", crate_path);

    let flags_clone = flags.clone();

    // Just run_with_crate
    let output = run_with_crate(crate_path, false, move |handle| {
        Box::pin(async move {
            trace!("Inside run_with_crate closure for ShowSubcommand::Crate");
            let cci_str = handle.show(&flags_clone).await?;
            if cci_str.trim().is_empty() && *flags_clone.show_items_with_no_data() {
                Ok("<no-data-for-crate>\n".to_string())
            } else {
                Ok(cci_str)
            }
        })
    })
    .await?;

    Ok(output)
}
