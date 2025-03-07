// ---------------- [ File: token-expansion-step/src/token_expansion_step.rs ]
crate::ix!();

pub enum TokenExpansionStep {
    ExtractAndCleanData,
    MapTokenToAxes {
        axes: Vec<Arc<dyn TokenExpansionAxis>>,
    },
    EnrichAndRephraseContent,
    ApplySpecificAdjustments,
    OutputTheJsonStructure,
}

impl TokenExpansionStep {

    pub fn vec_from_axes(axes: &[Arc<dyn TokenExpansionAxis>]) -> Vec<TokenExpansionStep> {

        // could provide more advanced behavior here. for now we just use the default for every variant
        TokenExpansionStep::default_steps_from_axes(axes)
    }

    pub fn default_steps_from_axes(axes: &[Arc<dyn TokenExpansionAxis>]) -> Vec<Self> {
        vec![
            TokenExpansionStep::ExtractAndCleanData,
            TokenExpansionStep::MapTokenToAxes { axes: axes.to_vec() },
            TokenExpansionStep::EnrichAndRephraseContent,
            TokenExpansionStep::ApplySpecificAdjustments,
            TokenExpansionStep::OutputTheJsonStructure,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            TokenExpansionStep::ExtractAndCleanData      => "Extract and Clean Data",
            TokenExpansionStep::MapTokenToAxes { .. }    => "Map Token to Axes",
            TokenExpansionStep::EnrichAndRephraseContent => "Enrich and Rephrase Content",
            TokenExpansionStep::ApplySpecificAdjustments => "Apply Specific Adjustments",
            TokenExpansionStep::OutputTheJsonStructure   => "Output the JSON Structure",
        }
    }

    pub fn ai_instructions(&self) -> String {
        match self {
            TokenExpansionStep::ExtractAndCleanData => "Carefully read and parse the token: ".to_string(),
            TokenExpansionStep::MapTokenToAxes { axes } => {

                let axes_descriptions = axes
                    .iter()
                    .map(|axis| {
                        format!(
                            "{} [{}]",
                            axis.axis_name(),
                            axis.axis_description()
                        )
                    }).collect::<Vec<_>>().join("\n");

                formatdoc!{
                    "
                        Reorganize the Extracted Data:

                        - Assign each piece of data to the appropriate axis based on its content.
                        - Ensure that each entry fits logically within its designated category.
                        - Each axis should be deep, detailed, and specific. Use optimally descriptive and useful language. Do not be too verbose.
                        - There should be at least twelve items per category. There may be more than twelve if you think it will be useful to provide more.

                        Axes to Use:

                        {}",
                        axes_descriptions
                }
            },
            TokenExpansionStep::EnrichAndRephraseContent => formatdoc!{
                "
                    Rephrase Entries:

                    - Ensure entries are concise and focused.
                    - Remove vague introductory phrases (e.g., avoid starting with \"Illustrates the...\"; instead, use direct details like \"The hanging green vines on the garden wall\").

                    Ensure Clarity and Consistency:

                    - Use clear, grammatically correct sentences.
                    - Maintain a consistent tone and style throughout.

                    Enrich Content:

                    - Add additional entries to each axis where appropriate to enhance depth and value.
                    - Ensure that the content is comprehensive and covers multiple facets of the token."
            },
            TokenExpansionStep::ApplySpecificAdjustments => formatdoc!{
                "
                    Focus Language:

                    - Use deliberate and precise language.
                    - Avoid vague verbs and keep the maximally intelligent and detail oriented audience in mind.
                    - Do not use modern cultural references or generically reference ideas which do not fit the overall aura of our setting.
                    "
            },
            TokenExpansionStep::OutputTheJsonStructure => formatdoc!{
                "
                    Present the Final JSON:

                    - Format the output as a JSON object.
                    - Include the token name and all the axes with their corresponding entries.
                    - The token name should be properly upper camel cased and placed under the `token_name` json key.
                
                    Ensure Proper Formatting:

                    - Use proper JSON syntax with keys and arrays.
                    - Ensure that all entries are correctly placed under their respective axes.

                    Your output should only consist of the JSON object. do *not* include a preamble or postamble to your response. We would like to be able to parse your response directly as JSON.
                    "

            },
        }
    }
}
