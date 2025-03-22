// ---------------- [ File: src/create_language_model_requests_at_agent_coordinate.rs ]
crate::ix!();

pub trait CreateLanguageModelQueryAtAgentCoordinate {

    fn create_language_model_query_at_agent_coordinate<X:IntoLanguageModelQueryString>(
        &self,
        model: &LanguageModelType, 
        coord: &AgentCoordinate, 
        input: &X

    ) -> String;
}

impl<T:GetSystemMessageAtAgentCoordinate> CreateLanguageModelQueryAtAgentCoordinate for T {

    fn create_language_model_query_at_agent_coordinate<X:IntoLanguageModelQueryString>(
        &self,
        model: &LanguageModelType, 
        coord: &AgentCoordinate, 
        input: &X

    ) -> String {

        let system_message = self.get_system_message_at_agent_coordinate(coord);
        let query_string   = input.into_language_model_query_string();

        formatdoc!{
            "
            {system_message}
            {query_string}
            "
        }
    }
}
