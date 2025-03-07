crate::ix!();

pub trait CreateLanguageModelRequestsAtAgentCoordinate {

    fn create_language_model_requests_at_agent_coordinate<X:IntoLanguageModelQueryString>(
        &self,
        model:     &LanguageModelType, 
        coord:     &AgentCoordinate, 
        inputs:    &[X]

    ) -> Vec<LanguageModelBatchAPIRequest>;
}

impl<T:GetSystemMessageAtAgentCoordinate> CreateLanguageModelRequestsAtAgentCoordinate for T {

    fn create_language_model_requests_at_agent_coordinate<X:IntoLanguageModelQueryString>(
        &self,
        model:     &LanguageModelType, 
        coord:     &AgentCoordinate, 
        inputs:    &[X]

    ) -> Vec<LanguageModelBatchAPIRequest>
    {
        let system_message = self.get_system_message_at_agent_coordinate(coord);

        let queries: Vec<String> = inputs.iter().map(|tok| tok.into_language_model_query_string()).collect();

        LanguageModelBatchAPIRequest::requests_from_query_strings(&system_message,model.clone(),&queries)
    }
}
