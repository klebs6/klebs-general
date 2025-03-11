// ---------------- [ File: src/rigorous_json.rs ]
crate::ix!();

pub enum RigorousJson {
    ExtractAndCleanData,
    GenerateResponseViaTheSchema {
        schema: serde_json::Value,
    },
    EnrichAndRephraseContent,
    ApplySpecificAdjustments,
    OutputTheJsonStructure,
}

impl RigorousJson {

    pub fn new(schema: &serde_json::Value) -> Vec<Self> {
        vec![
            RigorousJson::ExtractAndCleanData,
            RigorousJson::GenerateResponseViaTheSchema { schema: schema.clone() },
            RigorousJson::EnrichAndRephraseContent,
            RigorousJson::ApplySpecificAdjustments,
            RigorousJson::OutputTheJsonStructure,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            RigorousJson::ExtractAndCleanData                 => "Extract and Clean Data",
            RigorousJson::GenerateResponseViaTheSchema { .. } => "Generate Your Response via the Schema We Sent You",
            RigorousJson::EnrichAndRephraseContent            => "Enrich and Rephrase Content",
            RigorousJson::ApplySpecificAdjustments            => "Apply Specific Adjustments",
            RigorousJson::OutputTheJsonStructure              => "Output the JSON Structure",
        }
    }

    pub fn ai_instructions(&self) -> String {
        match self {
            RigorousJson::ExtractAndCleanData => "Carefully read and parse the information we sent you.".to_string(),
            RigorousJson::GenerateResponseViaTheSchema { schema } => {

                formatdoc!{
                    "
                        We want to generate all information items requested by our schema:

                        - Read the description of each field. Use it, along with the content we sent you to generate an appropriate value for each.
                        - Ensure that each generated field optimally serves its desired purpose.
                        - Each item should be deep, detailed, and specific. Use optimally descriptive and useful language. Do not be too verbose.

                        Schema to Use:

                        {}",
                        schema
                }
            },
            RigorousJson::EnrichAndRephraseContent => formatdoc!{
                "
                    Rephrase Entries:

                    - Ensure entries are concise and focused.
                    - Remove vague introductory phrases (e.g., avoid starting with \"Illustrates the...\"; instead, use direct details like \"The hanging green vines on the garden wall\").

                    Ensure Clarity and Consistency:

                    - Use clear, grammatically correct sentences.
                    - Maintain a consistent tone and style throughout.

                    Enrich Content:

                    - Add additional information where appropriate to enhance depth and value.
                    - If a mathematical, physical, or otherwise technical background is helpful, please provide it.
                    - You may include and interweave latin, sanskrit, and ancient greek vocabulary as you please.
                    - Ensure that the content is comprehensive and meets all requirements."
            },
            RigorousJson::ApplySpecificAdjustments => formatdoc!{
                "
                    Focus Language:

                    - Use deliberate and precise language.
                    - Avoid vague verbs and keep the maximally intelligent and detail oriented audience in mind.
                    - Do not use modern cultural references or generically reference ideas which do not fit the overall aura of our setting.
                    "
            },
            RigorousJson::OutputTheJsonStructure => formatdoc!{
                "
                    Present the Final JSON:

                    - Format the output as a JSON object.
                    - Include all fields properly named from the schema we sent you.
                
                    Ensure Proper Formatting:

                    - Use proper JSON syntax with keys and arrays.
                    - Ensure that all entries are correctly placed in their respective places.

                    Your output should only consist of the JSON object. do *not* include a preamble or postamble to your response. 
                    We would like to be able to parse your response directly as JSON.
                    "
            },
        }
    }
}
