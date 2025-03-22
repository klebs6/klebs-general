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

impl From<&InstructedLanguageModelAtCoordinate> for SystemMessageHeader {

    fn from(llm: &InstructedLanguageModelAtCoordinate) -> Self {

        let mut system_message_header = match llm.agent_coordinate() {
            Some(coord) => format!("{}",coord),
            None        => format!(""),
        };

        system_message_header.push_str("Notes:\n\n");

        for note in llm.instructions().iter() {
            for message in note.provide() {
                system_message_header.push_str(&format!("{}\n\n", message));
            }
        }

        SystemMessageHeader::new(&system_message_header)
    }
}
