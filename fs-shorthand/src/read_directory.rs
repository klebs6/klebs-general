crate::ix!();

#[inline] pub fn read_directory(dir: PathBuf) -> Result<Vec<PathBuf>,io::Error> {
    
    let mut paths = vec![];

    for entry in std::fs::read_dir(dir)? {

        let entry = entry?;
        let path  = entry.path();

        paths.push(path);
    }

    Ok(paths)
}
