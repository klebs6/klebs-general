// ---------------- [ File: src/instructed_language_model_at_coordinate.rs ]
crate::ix!();

#[derive(Clone,Builder,Getters,Debug)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct InstructedLanguageModelAtCoordinate {
    agent_coordinate: Option<AgentCoordinate>,
    instructions:     Vec<LanguageModelOutputFormatInstruction>,   
}

impl InstructedLanguageModelAtCoordinate {
    pub fn emit_detailed_json_objects(coord: &AgentCoordinate) -> Self {
        Self {
            agent_coordinate: Some(coord.clone()),
            instructions:     language_model_output_format_instructions::generate_detailed_json_objects(),
        }
    }
}
