crate::ix!();

pub fn pdflatex(output_directory: &str, output_path: &str) -> Result<(),std::io::Error> {

    // Run pdflatex command with output directory specified
    Command::new("pdflatex")
        .arg(format!("-output-directory={}", output_directory))
        .arg(&output_path)
        .status()?;

    Ok(())
}
