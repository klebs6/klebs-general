// ---------------- [ File: src/language_model_format_instruction.rs ]
crate::ix!();

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum LanguageModelOutputFormatInstruction {
    AvoidVagueness,
    ProvideOutputAsValidJson,
}

impl LanguageModelOutputFormatInstruction {

    pub fn provide(&self) -> Vec<&'static str> {
        match self {
            LanguageModelOutputFormatInstruction::AvoidVagueness => vec![
                "Steer clear of vague sweeping generalizations. Instead focus on concrete, direct, specific details and descriptive language.",
                "Don't ever use the term `narrative` or anything similarly vague. Please steer clear of all vague concepts",
            ],
            LanguageModelOutputFormatInstruction::ProvideOutputAsValidJson => vec![
                "Please provide the output as a valid JSON object. Do not include any explanations, code block markers, or additional text. The JSON should start with `{` and end with `}`.",
                "The JSON data should be free of typos like naked double quotes within strings and extra or missing punctuation marks",
            ],
        }
    }
}

pub mod language_model_output_format_instructions {
    use super::*;
    pub fn generate_detailed_json_objects() -> Vec<LanguageModelOutputFormatInstruction> {
        vec![
            LanguageModelOutputFormatInstruction::AvoidVagueness,
            LanguageModelOutputFormatInstruction::ProvideOutputAsValidJson,
        ]
    }
}
