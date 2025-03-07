crate::ix!();

/// The improved LanguageModelTokenExpander, now with no `pub` fields. Instead, we rely on
/// `getset` to provide getters (and optionally setters) and `derive_builder` for
/// constructing robustly. This struct implements `LanguageModelBatchWorkflow` to unify
/// your batch processing logic under a trait-based approach.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct LanguageModelTokenExpander<T: CreateLanguageModelRequestsAtAgentCoordinate> {
    client:                         Arc<OpenAIClientHandle>,
    workspace:                      Arc<BatchWorkspace>,
    model:                          LanguageModelType,
    language_model_request_creator: Arc<T>,
    inputs:                         Vec<<Self as LanguageModelBatchWorkflow>::Seed>,
    unseen_inputs:                  Vec<<Self as LanguageModelBatchWorkflow>::Seed>,
    language_model_requests:        Vec<LanguageModelBatchAPIRequest>,
}

unsafe impl<T:CreateLanguageModelRequestsAtAgentCoordinate> Send for LanguageModelTokenExpander<T> {}
unsafe impl<T:CreateLanguageModelRequestsAtAgentCoordinate> Sync for LanguageModelTokenExpander<T> {}

impl<T:CreateLanguageModelRequestsAtAgentCoordinate> LanguageModelTokenExpander<T> {

    pub async fn new(
        product_root:        impl AsRef<Path>,
        model:               LanguageModelType, 
        language_model_request_creator: Arc<T>

    ) -> Result<Self,TokenExpanderError> {

        info!("creating LanguageModelTokenExpander with model: {:#?}", model);

        Ok(Self {
            client:    OpenAIClientHandle::new(),
            workspace: BatchWorkspace::new_in(product_root).await?,
            model,
            language_model_request_creator,
            unseen_inputs: vec![],
            language_model_requests:        vec![],
        })
    }

    delegate!{
        to self.workspace {
            pub fn input_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn batch_expansion_error_log_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn output_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn error_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
            pub fn token_expansion_path(&self,token_name: &<Self as LanguageModelBatchWorkflow>::Seed) -> PathBuf;
        }
    }

    /// Internal helper from your original code. Identifies newly seen tokens.
    #[tracing::instrument(level="info", skip_all)]
    pub fn calculate_unseen_inputs(&mut self, inputs: &[CamelCaseTokenWithComment]) {
        self.unseen_inputs = calculate_unseen_input_tokens(&self.workspace, inputs);
        info!("Unseen input tokens calculated:");
        for token in &self.unseen_inputs {
            info!("{}", token);
        }
    }
}

/// Here we implement the trait that organizes all batch-processing stages.
#[async_trait]
impl<T: CreateLanguageModelRequestsAtAgentCoordinate> LanguageModelBatchWorkflow for LanguageModelTokenExpander<T> {

    type Seed  = CamelCaseTokenWithComment;
    type Error = TokenExpanderError;

    async fn maybe_finish_processing_uncompleted_batches(
        &self,
        expected_content_type: ExpectedContentType
    ) -> Result<(), Self::Error> {
        info!("Finishing uncompleted batches if any remain.");

        let mut batch_triples = self.workspace().gather_all_batch_triples().await?;

        info!("Reconciling unprocessed batch files in the work directory");
        for triple in &mut batch_triples {
            reconcile_unprocessed_batch_triple(triple, &self.client, expected_content_type).await?;
        }
        Ok(())
    }

    fn compute_language_model_requests(
        &mut self,
        model:  &LanguageModelType,
        inputs: &[Self::Seed]
    ) {
        trace!("Computing GPT requests from newly provided tokens...");
        self.calculate_unseen_inputs(inputs);
        self.language_model_requests = self.language_model_request_creator.create_language_model_requests(
            "YOU_ARE_HERE",
            model.clone(),
            &self.unseen_inputs
        );
    }

    fn construct_batches(
        &self
    ) -> Enumerate<Chunks<'_, LanguageModelBatchAPIRequest>> {
        const GPT_REQUESTS_PER_BATCH: usize = 80;
        let mut batches = self.language_model_requests.chunks(GPT_REQUESTS_PER_BATCH).enumerate();

        // If there's exactly 1 chunk, and it's under 32, panic:
        if batches.len() == 1 {
            let only_batch_len = batches.nth(0).unwrap().1.len();
            if only_batch_len < 32 {
                panic!(
                    "attempting to construct a trivially small batch of size {}. \
                     are you sure you want to do this?",
                    only_batch_len
                );
            }
        }
        info!(
            "Constructing {} batch(es), each with max {} items",
            batches.len(),
            GPT_REQUESTS_PER_BATCH
        );

        // Rebuild the enumerator, because we consumed it with nth(0).
        self.language_model_requests.chunks(GPT_REQUESTS_PER_BATCH).enumerate()
    }

    async fn process_batch_requests(
        &self,
        batch_requests: &[LanguageModelBatchAPIRequest]
    ) -> Result<(), Self::Error> {
        info!("Processing {} batch request(s)", batch_requests.len());
        let mut triple = BatchFileTriple::new_with_requests(batch_requests, self.workspace.clone())?;
        let execution_result = fresh_execute_batch_processing(&mut triple, &self.client).await?;
        process_batch_output_and_errors(&*self.workspace, &execution_result).await?;
        triple.move_all_to_done().await?;
        Ok(())
    }

    async fn execute_workflow(
        &mut self,
        model: &LanguageModelType,
        inputs: &[Self::Seed]
    ) -> Result<(), Self::Error> {
        info!("Beginning full GPT batch workflow execution");
        let expected_content_type = ExpectedContentType::Json;

        self.maybe_finish_processing_uncompleted_batches(expected_content_type).await?;
        self.compute_language_model_requests(model, inputs);

        let batches = self.construct_batches();
        for (batch_idx, batch_requests) in batches {
            info!("Processing batch #{}", batch_idx);
            self.process_batch_requests(batch_requests).await?;
        }
        Ok(())
    }
}
