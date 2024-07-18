crate::ix!();

pub fn maybe_remove_files<P: AsRef<Path>>(files: &Vec<P>, gate: bool)
-> std::io::Result<()>
{
    if gate {

        for file in files.iter() {

            std::fs::remove_file(file)?;
        }
    }

    Ok(())
}
