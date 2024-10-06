crate::ix!();

/// Returns the `PathBuf` to the parent directory's `Cargo.toml` file.
///
/// This function will panic if the parent directory's `Cargo.toml` does not exist.
///
/// # Panics
///
/// - Panics if it fails to get the current directory.
/// - Panics if the parent directory's `Cargo.toml` does not exist.
///
/// # Examples
///
/// ```no_run
/// use scan_crate_for_typedefs::*;
///
/// let path = parent_cargo_toml();
/// ```
pub fn parent_cargo_toml() -> PathBuf {

    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    let res = current_dir.join("..").join("Cargo.toml").canonicalize().unwrap();

    if !res.exists() {

        panic!("wtf {:?} DNE", res);
    }

    res
}

/// Returns the `PathBuf` to the current directory's `Cargo.toml` file.
///
/// # Panics
///
/// - Panics if it fails to get the current directory.
///
/// # Examples
///
/// ```no_run
/// use scan_crate_for_typedefs::*;
///
/// let path = current_cargo_toml();
/// ```
pub fn current_cargo_toml() -> PathBuf {

    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    current_dir.join("Cargo.toml")
}

/// Retrieves the crate name from a given `Cargo.toml` file path.
///
/// # Errors
///
/// Returns an `io::Result` which is an `Err` if:
///
/// - The `Cargo.toml` file could not be opened.
/// - The content of `Cargo.toml` could not be parsed.
/// - The `name` field is not found in the `[package]` section.
///
/// # Examples
///
/// ```no_run
/// use scan_crate_for_typedefs::*;
///
/// let name = get_crate_name_from_cargo_toml("path/to/Cargo.toml").unwrap();
/// ```
pub fn get_crate_name_from_cargo_toml(path: &str) -> io::Result<String> {
    // Read the content of the Cargo.toml file
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Parse the TOML content
    let parsed: toml::Value = content.parse().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse Cargo.toml: {}", e),
        )
    })?;

    // Extract the crate name
    let name = parsed
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|name| name.as_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to find the 'name' field in the [package] section of Cargo.toml",
            )
        })?;

    Ok(name.to_string())
}

/// Retrieves the parent directory path from a given `Cargo.toml` file path.
///
/// # Errors
///
/// Returns an `io::Result` which is an `Err` if:
///
/// - The `path` could not be converted to a `Path` object.
/// - The parent directory path could not be converted to a `String`.
///
/// # Examples
///
/// ```rust
/// use scan_crate_for_typedefs::*;
///
/// let parent_dir = get_parent_directory_from_cargo_toml_path("path/to/Cargo.toml").unwrap();
/// ```
///
pub fn get_parent_directory_from_cargo_toml_path(path: &str) -> io::Result<String> {
    // Create a Path from the String
    let path = Path::new(path);

    // Use the parent method to find the parent directory
    let parent_path = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Failed to find the parent directory",
        )
    })?;

    // Convert the parent directory path to a String
    let parent_path_str = parent_path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Failed to convert the parent directory path to a String",
        )
    })?;

    Ok(parent_path_str.to_string())
}
