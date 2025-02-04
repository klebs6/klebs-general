crate::ix!();

/// Ensures the path does not point to an existing directory.
pub fn validate_not_dir(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        let msg = format!("Refusing to create file at {:?}, path is a directory", path);
        error!("{}", msg);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg));
    }
    Ok(())
}

