crate::ix!();

pub fn maybe_create_directory<P: AsRef<Path>>(outdir: P)
-> std::io::Result<()>
{
    if !outdir.as_ref().exists() {
        std::fs::create_dir(outdir.as_ref())?;
    }

    Ok(())
}
