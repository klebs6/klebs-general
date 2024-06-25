crate::ix!();

#[derive(StructOpt, Debug)]
#[structopt(name = "resume_generator")]
pub struct Cli {
    /// Output filename
    #[structopt(long)]
    output_filename: String,

    /// Output directory
    #[structopt(long)]
    output_directory: String,
}

impl Cli {

    pub fn output_filename(&self) -> &str {
        &self.output_filename
    }

    pub fn output_directory(&self) -> &str {
        &self.output_directory
    }
}
