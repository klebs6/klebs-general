crate::ix!();

/// Saves the `WorkspaceTypes` instance to a JSON file at the specified path.
///
/// # Arguments
///
/// - `workspace_types`: A reference to the `WorkspaceTypes` instance to save.
/// - `path`: The path to the file where the `WorkspaceTypes` instance will be saved.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of the operation.
pub fn save_workspace_types<P: AsRef<Path>>(workspace_types: &WorkspaceTypes, path: P) 
    -> std::io::Result<()> 
{
    workspace_types.save_to_json(path)
}

/// Loads a `WorkspaceTypes` instance from a JSON file at the specified path.
///
/// # Arguments
///
/// - `path`: The path to the JSON file to load the `WorkspaceTypes` instance from.
///
/// # Returns
///
/// Returns a `Result` containing the loaded `WorkspaceTypes` instance on success,
/// or an error on failure.
pub fn load_workspace_types<P: AsRef<Path>>(path: P) -> std::io::Result<WorkspaceTypes> {
    WorkspaceTypes::load_from_json(path)
}

/// Prints the content of the file located at the specified path to the standard output.
///
/// # Arguments
///
/// - `path`: The path to the file whose content will be printed.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of the operation.
pub fn cat_file_to_screen<P: AsRef<Path>>(path: P) -> io::Result<()> {

    let file   = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }

    Ok(())
}
