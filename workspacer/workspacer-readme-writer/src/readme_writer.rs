// ---------------- [ File: workspacer-readme-writer/src/readme_writer.rs ]
crate::ix!();

#[derive(Serialize,Deserialize,Builder,MutGetters,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub",get_mut="pub")]
pub struct AiReadmeQuery {
    #[builder(default)]
    query_text: String,

    #[builder(default)]
    instructions: String,
}

#[derive(Serialize,Deserialize,Builder,MutGetters,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub",get_mut="pub")]
pub struct AiReadmeResponse {
    #[builder(default)]
    proposed_readme_text: String,

    #[builder(default)]
    commentary: Option<String>,
}

/// A stub async function that simulates sending the queries to an AI agent.
/// In real code, you would call your LLM or some API. Here, it just returns
/// a placeholder response for each query.
#[tracing::instrument(level="trace", skip_all)]
pub async fn send_readme_queries_to_ai(
    queries: &[AiReadmeQuery]
) -> Result<Vec<AiReadmeResponse>, CrateError> {
    trace!("Entering send_readme_queries_to_ai with {} queries", queries.len());
    let mut results = Vec::new();

    for (i, q) in queries.iter().enumerate() {
        debug!(
            "Simulating AI call for query #{} => query_text: {:?}",
            i,
            q.query_text()
        );

        let resp = AiReadmeResponseBuilder::default()
            .proposed_readme_text(format!(
                    "# Automated README\n\n(Stub AI Output)\n\nOriginal Instructions:\n{}\n\nConsolidated Items:\n{}",
                    q.instructions(),
                    q.query_text()
            ))
            .commentary(Some("This is a mocked response from the AI agent.".to_string()))
            .build()
            .unwrap();

        results.push(resp);
    }

    info!("Exiting send_readme_queries_to_ai; returning {} responses", results.len());
    Ok(results)
}

/*
pub struct TokenExpansionSystemMessageBuilder {
    you_are_here:          String,
    system_message_header: String,
    steps:                 Vec<TokenExpansionStep>,
}

impl TokenExpansionSystemMessageBuilder {

    pub fn new(you_are_here: &str, expander: &impl TokenExpander) -> Self {

        let system_message_goal = expander.system_message_goal();

        Self {
            you_are_here:          you_are_here.to_string(),
            system_message_header: Self::build_system_message_header(you_are_here,&system_message_goal),
            steps:                 TokenExpansionStep::vec_from_axes(&expander.axes()),
        }
    }

    pub fn build_system_message_header(you_are_here: &str, system_message_goal: &str) -> String {

        const SYSTEM_MESSAGE_NOTES: &[&'static str] = &[
            "Steer clear of vague sweeping generalizations. Instead focus on concrete, direct, specific details and descriptive language.",
            "Don't ever use the term `narrative` or anything similarly vague. Please steer clear of all vague concepts",
            "Please provide the output as a valid JSON object. Do not include any explanations, code block markers, or additional text. The JSON should start with `{` and end with `}`.",
            "The JSON data should be free of typos like naked double quotes within strings and extra or missing punctuation marks",
        ];

        let mut system_message = format!("You are here: {}", you_are_here);
        system_message.push_str(&system_message_goal);
        system_message.push_str("Notes:\n\n");
        for note in SYSTEM_MESSAGE_NOTES.iter() {
            system_message.push_str(&format!("{}\n\n", note));
        }
        system_message
    }

    pub fn build(&self) -> String {

        const GLOBAL_INSTRUCTIONS: &'static str 
            = "Convert the provided token into a JSON representation, restructured according to the specified axes. 
            Follow these steps meticulously to ensure consistency and completeness:";

        // Build the query string using the token and the structured data
        let mut query = format!("{}\n\nInstructions:\n\n{}\n\n", self.system_message_header, GLOBAL_INSTRUCTIONS);

        for (i, step) in self.steps.iter().enumerate() {
            query.push_str(&format!("Step {}: {}\n\n{}", i + 1, step.name(), step.ai_instructions()));
        }

        query
    }
}

pub fn create_gpt_requests(
    you_are_here: &str, 
    expander:     &impl TokenExpander, 
    model:        GptModelType, 
    input_tokens: &[CamelCaseTokenWithComment]

) -> Vec<GptBatchAPIRequest>
{
    let system_message = TokenExpansionSystemMessageBuilder::new(you_are_here,expander).build();

    let queries: Vec<String> = input_tokens.iter().map(|tok| tok.clone().into()).collect();

    GptBatchAPIRequest::requests_from_query_strings(&system_message,model,&queries)
}
*/
