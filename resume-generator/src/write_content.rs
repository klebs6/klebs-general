crate::ix!();

pub fn write_content(content: &str, path: &str) -> Result<(),std::io::Error> {
    let mut file = File::create(&path).expect("Could not create file");
    file.write_all(content.as_bytes()).expect("Could not write to file");
    Ok(())
}
