crate::ix!();

pub struct AiTomlWriter {

}

impl CreateGptRequests for AiTomlWriter {

    type Seed = ConsolidatedCrateInterface;

    fn create_gpt_requests(
        you_are_here: &str, 
        model:  GptModelType, 
        inputs: &[Self::Seed]
    ) -> Vec<GptBatchAPIRequest> {

        let system_message = todo!();;

        let queries: Vec<String> = todo!();

        GptBatchAPIRequest::requests_from_query_strings(&system_message,model,&queries)
    }
}
