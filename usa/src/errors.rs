crate::ix!();

#[derive(Error,Debug, Clone)]
#[error("Bad input! {input}")]
pub struct BadInput {
    input: String,
}

impl BadInput {

    pub fn bad(input: &str) -> Self {
        Self {
            input: input.to_string()
        }
    }
}
