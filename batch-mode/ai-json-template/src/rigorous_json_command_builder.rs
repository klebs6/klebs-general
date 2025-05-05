crate::ix!();

pub struct RigorousJsonCommandBuilder;

impl RigorousJsonCommandBuilder {

    pub fn instructions<T:AiJsonTemplate>() -> String {
        let stages = RigorousJsonCommandBuilderStage::all();
        let mut x = String::new();
        for stage in stages {
            let schema_template = T::to_template();
            x.push_str(&stage.ai_instructions(&schema_template));
            x.push_str("\n");
        }
        x
    }

    pub fn instructions_with_justification<T:AiJsonTemplateWithJustification>() -> String {
        let stages = RigorousJsonCommandBuilderStage::all();
        let mut x = String::new();
        for stage in stages {
            let schema_template = T::to_template_with_justification();
            x.push_str(&stage.ai_instructions(&schema_template));
            x.push_str("\n");
        }
        x
    }
}
