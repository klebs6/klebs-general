// ---------------- [ File: batch-mode-batch-scribe/src/write_seed_manifest.rs ]
crate::ix!();

pub trait SeedManifestEntry {
    fn custom_id(&self) -> String;
}

#[instrument(level = "trace", skip(manifest_path, requests))]
pub fn write_seed_manifest<T>(
    manifest_path: &Path,
    requests: &[T],
) -> Result<(), std::io::Error> 
where
    T: SeedManifestEntry,
{
    use std::fs;
    use std::io::Write;

    trace!("Writing seed manifest to {:?}", manifest_path);

    fs::create_dir_all(manifest_path.parent().unwrap())?;
    let mut file = fs::File::create(manifest_path)?;

    for request in requests {
        let entry = serde_json::json!({
            "custom_id": request.custom_id(),
        });

        writeln!(file, "{}", entry)?;
        debug!("Manifest entry written: {}", entry);
    }

    info!("Seed manifest written successfully to {:?}", manifest_path);
    Ok(())
}
