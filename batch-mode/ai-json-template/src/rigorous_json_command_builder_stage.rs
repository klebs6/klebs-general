// ---------------- [ File: ai-json-template/src/rigorous_json_command_builder_stage.rs ]
crate::ix!();

pub enum RigorousJsonCommandBuilderStage {
    ExtractAndCleanData,
    GenerateResponseViaTheSchema,
    OptimizeContent,
    ApplySpecificAdjustments,
    OutputTheJsonStructure,
}

impl RigorousJsonCommandBuilderStage {

    pub fn all() -> Vec<Self> {
        vec![
            RigorousJsonCommandBuilderStage::ExtractAndCleanData,
            RigorousJsonCommandBuilderStage::GenerateResponseViaTheSchema,
            RigorousJsonCommandBuilderStage::OptimizeContent,
            RigorousJsonCommandBuilderStage::ApplySpecificAdjustments,
            RigorousJsonCommandBuilderStage::OutputTheJsonStructure,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            RigorousJsonCommandBuilderStage::ExtractAndCleanData                 => "Extract and Clean Data",
            RigorousJsonCommandBuilderStage::GenerateResponseViaTheSchema { .. } => "Generate Your Response via the Schema We Sent You",
            RigorousJsonCommandBuilderStage::OptimizeContent                     => "Ensure Content is Optimal",
            RigorousJsonCommandBuilderStage::ApplySpecificAdjustments            => "Apply Specific Adjustments",
            RigorousJsonCommandBuilderStage::OutputTheJsonStructure              => "Output the JSON Structure",
        }
    }

    pub fn ai_instructions(&self, schema_template: &serde_json::Value) -> String {
        match self {
            RigorousJsonCommandBuilderStage::ExtractAndCleanData => "Carefully read and parse the information we sent you.\n".to_string(),
            RigorousJsonCommandBuilderStage::GenerateResponseViaTheSchema => {

                formatdoc!{
                    "
                        We want to generate all information items requested by our schema:

                        - Read the description of each field. Use it, along with the content we sent you to generate an appropriate value for each.
                        - Ensure that each generated field optimally serves its desired purpose.
                        - Each item should be deep, detailed, and specific. Use optimally descriptive and useful language. Do not be too verbose.
                        - Do not add any extra keys to the generated object. Generate precisely what we ask of you: no more, no less.
                        - For fields which are of an enum type: pick *exactly one variant*. Do not generate any information for the unselected variants.
                        - For array fields: supply a JSON list [ ... ] of items, each conforming to the correct item type.
                        - For map fields: Provide a JSON object {{ \"key_as_string\": <value>, ... }}. Keys must be valid JSON strings. Keys and Values must conform to the key and value schemas we provide to you. Fill all subfields of each and every map value.
                        - If we ask you to justify a map field, provide your justification/confidence score for the entire map. Do not attempt to separately justify each map element.
                        - Likewise, if we ask you to justify an array field, provide your justification/confidence score for the entire array. Do not attempt to separately justify each element.
                        - Provide numeric fields as real JSON numbers (not strings).
                        - For string fields: provide the value precisely as a string literal. If optional, you may set it to null.
                        - If we ask you to justify or provide confidence scores for the fields of a structure, please do so carefully. For each field we ask you to justify, please explain or justify your choice clearly. Confidence scores must be specified as real JSON numbers in the range `[0..1]`.
                        - If we ask to you generate an optional field, either fill it or set it to null.
                        - If we ask you to justify your choice of an enum variant, please explain your choice clearly. Do not justify your decision to leave a variant unselected. Provide your justification as a JSON string. Provide your confidence score as a real JSON number in the range `[0..1]`.
                        - In terms of the JSON format we would like you to provide, the object itself will be deserialied based on the following schema. 
                        - We begin exactly at the top: the data should not be nested under a parent named 'fields' or anything similar.

                        ## Here is the schema to use:

                        {:#?}
                    ",
                        schema_template
                }
            },
            RigorousJsonCommandBuilderStage::OptimizeContent => formatdoc!{
                "
                    ## Please keep the following in mind:

                    Entry phrasing:

                    - Ensure entries are concise and focused.
                    - Ensure there are no vague phrases (e.g., avoid starting with \"Illustrates the something something...\"; instead, use direct details like \"The hanging green vines on the garden wall\").

                    Ensure Clarity and Consistency:

                    - Use clear, grammatically correct sentences.
                    - Maintain a consistent tone and style throughout.

                    Enrich Content:

                    - Communicate additional information where appropriate to enhance depth and value.
                    - If a mathematical, physical, or otherwise technical background is helpful, please provide it.
                    - Ensure that the content is comprehensive and meets all requirements.
                    "
            },
            RigorousJsonCommandBuilderStage::ApplySpecificAdjustments => formatdoc!{
                "
                    Focus Language:

                    - Use deliberate and precise language.
                    - Avoid vague verbs and keep the maximally intelligent and detail oriented audience in mind. 
                    - We explicitly ban the words `hush`, `subtle`, `mythic`, and `aura`. Please keep your vocabulary choices rigorous.
                    - Do not use modern cultural references 
                    - Do not generically reference ideas which do not belong in the global environment.
                    "
            },
            RigorousJsonCommandBuilderStage::OutputTheJsonStructure => formatdoc!{
                "
                    ## Present the Final JSON:

                    - Format the output as a JSON object.
                    - Include all fields properly named from the schema we sent you.
                
                    Ensure Proper Formatting:

                    - Use proper JSON syntax with keys and arrays.
                    - Ensure that all entries are correctly placed in their respective places.

                    Your output should only consist of the JSON object. do *not* include a preamble or postamble to your response. 

                    We would like to be able to parse your response directly as JSON."
            },
        }
    }
}
