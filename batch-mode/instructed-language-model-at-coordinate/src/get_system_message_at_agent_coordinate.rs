// ---------------- [ File: instructed-language-model-at-coordinate/src/get_system_message_at_agent_coordinate.rs ]
crate::ix!();

pub trait GetSystemMessageAtAgentCoordinate {

    fn get_system_message_at_agent_coordinate(&self, coord: &AgentCoordinate) -> String;
}

impl<T: TokenExpander> GetSystemMessageAtAgentCoordinate for T {

    fn get_system_message_at_agent_coordinate(&self, coord: &AgentCoordinate) -> String {

        let llm   = InstructedLanguageModelAtCoordinate::emit_detailed_json_objects(coord);
        let steps = TokenExpansionStep::vec_from_axes(&self.axes());

        let message_header = SystemMessageHeader::from(&llm);

        const GLOBAL_INSTRUCTIONS: &'static str 
            = "Convert the provided token into a JSON representation, restructured according to the specified axes. 
            Follow these steps meticulously to ensure consistency and completeness:";

        let mut system_message = format!{
            "{}\n\nInstructions:\n\n{}\n\n", 
            message_header, 
            GLOBAL_INSTRUCTIONS
        };

        for (i, step) in steps.iter().enumerate() {
            system_message.push_str(&format!("Step {}: {}\n\n{}", i + 1, step.name(), step.ai_instructions()));
        }

        system_message
    }
}
