crate::ix!();

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SystemMessageHeader(String);

impl SystemMessageHeader {
    pub fn get(self) -> String {
        self.0
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

        SystemMessageHeader(system_message_header)
    }
}
