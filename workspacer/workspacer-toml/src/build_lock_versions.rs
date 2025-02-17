crate::ix!();

/// A shared helper that reads `Cargo.lock` from `root` and builds a map of crate->versions.
pub async fn build_lock_versions<P>(
    root: &P
) -> Result<BTreeMap<String, BTreeSet<cargo_lock::Version>>, CrateError>
where
    P: AsRef<Path> + Send + Sync,
{
    let lock_path = root.as_ref().join("Cargo.lock");
    if !lock_path.exists() {
        return Err(CrateError::FileNotFound {
            missing_file: lock_path,
        });
    }

    let lockfile_str = fs::read_to_string(&lock_path)
        .await
        .map_err(|e| CrateError::IoError {
            io_error: Arc::new(e),
            context:  format!("Failed to read Cargo.lock at {:?}", lock_path),
        })?;

    let lockfile = cargo_lock::Lockfile::from_str(&lockfile_str).map_err(|e| {
        CrateError::LockfileParseFailed {
            path: lock_path.clone(),
            message: format!("{e}"),
        }
    })?;

    let mut map: BTreeMap<String, BTreeSet<cargo_lock::Version>> = BTreeMap::new();
    for cargo_lock::Package { name, version, .. } in &lockfile.packages {
        map.entry(name.as_str().to_owned())
            .or_default()
            .insert(version.clone());
    }
    debug!("build_lock_versions: created map with {} crates", map.len());
    Ok(map)
}
