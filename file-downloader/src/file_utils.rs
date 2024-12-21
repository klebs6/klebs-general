crate::ix!();

pub fn base_filename_without_extension(fname: impl AsRef<Path>, extension: &str) 
    -> String 
{
    // Extract base filename from download link
    let fstr = fname.as_ref().to_str().unwrap_or("");
    fstr.replace(extension, "")
}

pub fn filename(download_link: &str) 
-> PathBuf 
{
    PathBuf::from(download_link.split('/').last().unwrap())
}
