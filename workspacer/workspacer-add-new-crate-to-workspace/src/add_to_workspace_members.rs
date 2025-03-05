// ---------------- [ File: src/add_to_workspace_members.rs ]
crate::ix!();

#[async_trait]
pub trait AddToWorkspaceMembers<P> 
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + Clone + 'static,
{
    async fn add_to_workspace_members(
        &self,
        new_crate_path: &P,
    ) -> Result<(), WorkspaceError>;
}

#[async_trait]
impl<P,H> AddToWorkspaceMembers<P> for Workspace<P,H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + Clone + 'static,
    H: CrateHandleInterface<P> + Send + Sync + 'static,
{
    async fn add_to_workspace_members(
        &self,
        new_crate_path: &P,
    ) -> Result<(), WorkspaceError> {
        let top_cargo = self.as_ref().join("Cargo.toml");
        if !top_cargo.exists() {
            warn!("No top-level Cargo.toml found => skipping workspace membership update");
            return Ok(());
        }

        let contents = fs::read_to_string(&top_cargo).await.map_err(|e| {
            WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: "reading top-level Cargo.toml".into()
            }
        })?;

        let mut doc = contents.parse::<TomlEditDocument>().map_err(|toml_err| {
            CargoTomlError::TomlEditError {
                cargo_toml_file: top_cargo.clone(),
                toml_parse_error: toml_err,
            }
        })?;

        if doc.get("workspace").is_none() {
            warn!("top-level Cargo.toml lacks [workspace]; skipping membership update");
            return Ok(());
        }

        let rel_path = new_crate_path.as_ref().strip_prefix(self.as_ref())
            .unwrap_or(new_crate_path.as_ref())
            .to_string_lossy()
            .replace("\\", "/"); // for windows

        // Ensure [workspace].members is an array
        if doc["workspace"]["members"].is_none() {
            doc["workspace"]["members"] = TomlEditItem::Value(TomlEditValue::Array(TomlEditArray::new()));
        }

        let members_arr = doc["workspace"]["members"]
            .as_array_mut()
            .ok_or_else(|| {
                CargoTomlError::CargoTomlWriteError(
                    CargoTomlWriteError::WriteWorkspaceMember {
                        io: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, 
                            "workspace.members not an array?")),
                    }
                )
            })?;

        // If it's not already in the array, push it
        if !members_arr.iter().any(|itm| itm.as_str() == Some(&rel_path)) {
            members_arr.push(&rel_path);
            let updated = doc.to_string();
            fs::write(&top_cargo, updated).await.map_err(|e| {
                WorkspaceError::IoError {
                    io_error: Arc::new(e),
                    context: format!("writing top-level Cargo.toml after adding member={}", rel_path)
                }
            })?;
        } else {
            debug!("new crate path='{}' is already in workspace.members => skipping", rel_path);
        }
        Ok(())
    }
}
