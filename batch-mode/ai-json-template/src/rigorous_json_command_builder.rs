// ---------------- [ File: ai-json-template/src/rigorous_json_command_builder.rs ]
crate::ix!();

pub struct RigorousJsonCommandBuilder;

impl RigorousJsonCommandBuilder {

    pub fn instructions<T:AiJsonTemplate>() -> String {
        let stages = RigorousJsonCommandBuilderStage::all();
        let mut x: Vec<String> = Vec::new();
        for stage in stages {
            let schema_template = T::to_template();
            x.push(stage.ai_instructions(&schema_template));
        }
        x.join("\n")
    }

    pub fn instructions_with_justification<T:AiJsonTemplateWithJustification>() -> String {
        let stages = RigorousJsonCommandBuilderStage::all();
        let mut x: Vec<String> = Vec::new();
        for stage in stages {
            let schema_template = T::to_template_with_justification();
            x.push(stage.ai_instructions(&schema_template));
        }
        x.join("\n")
    }
}
