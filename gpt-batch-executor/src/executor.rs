crate::ix!();

pub trait OpenAIConfigInterface = async_openai::config::Config;

pub struct GptBatchExecutor<C: OpenAIConfigInterface> {
    client: Client<C>,
}

impl<C: OpenAIConfigInterface> GptBatchExecutor<C> {
    pub fn new(client: Client<C>) -> Self {
        Self { client }
    }

    pub async fn execute_batch(
        &self,
        requests: &[GptBatchAPIRequest],
        batch_input_filename:  impl AsRef<Path> + std::fmt::Debug,
        batch_output_filename: impl AsRef<Path> + std::fmt::Debug,
        batch_error_filename:  impl AsRef<Path> + std::fmt::Debug,
    ) -> Result<Vec<BatchRequestOutput>, Box<dyn std::error::Error>> {

        // Create input file
        create_batch_input_file(&requests,&batch_input_filename)?;

        // Upload file
        let input_file_id = upload_batch_file(&self.client, &batch_input_filename).await?;

        // Create batch
        let batch = create_batch(&self.client, input_file_id).await?;
        let batch_id = batch.id.clone();

        // Wait for completion
        let completed_batch = wait_for_batch_completion(&self.client, batch_id).await?;

        // Download output file
        let outputs = if let Some(output_file_id) = completed_batch.output_file_id {
            download_output_file(&self.client, output_file_id, &batch_output_filename).await?;
            parse_batch_output(&batch_output_filename)?
        } else {
            Vec::new()
        };

        // Handle errors if any
        if let Some(error_file_id) = completed_batch.error_file_id {
            download_output_file(&self.client, error_file_id, &batch_error_filename).await?;
            let _errors = parse_batch_output(&batch_error_filename)?;
            eprintln!("Some requests failed. See '{:?}' for details.", &batch_error_filename);
        }

        Ok(outputs)
    }
}

pub async fn wait_for_batch_completion<C: OpenAIConfigInterface>(
    client: &Client<C>,
    batch_id: String,
) -> Result<Batch, OpenAIError> {
    loop {
        let batch = client.batches().retrieve(&batch_id).await?;
        match batch.status {
            BatchStatus::Completed => return Ok(batch),
            BatchStatus::Failed => {
                return Err(OpenAIError::ApiError(OpenAIApiError {
                    message: "Batch failed".to_string(),
                    r#type: None,
                    param: None,
                    code: None,
                }))
            }
            _ => {
                println!("Batch status: {:?}", batch.status);
                sleep(Duration::from_secs(30)).await; // Wait before checking again
            }
        }
    }
}

pub async fn upload_batch_file<C: OpenAIConfigInterface>(
    client: &Client<C>,
    file_path: impl AsRef<Path>,
) -> Result<String, OpenAIError> {
    let create_file_request = CreateFileRequest {
        file: file_path.into(),
        purpose: FilePurpose::Batch,
    };

    let file = client.files().create(create_file_request).await?;
    Ok(file.id)
}

pub fn parse_batch_output(
    file_path: impl AsRef<Path>,
) -> Result<Vec<BatchRequestOutput>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut outputs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let output: BatchRequestOutput = serde_json::from_str(&line)?;
        outputs.push(output);
    }

    Ok(outputs)
}

pub async fn download_output_file<C: OpenAIConfigInterface>(
    client: &Client<C>,
    file_id: String,
    output_path: impl AsRef<Path>,
) -> Result<(), OpenAIError> {
    let file_content = client.files().content(&file_id).await?;
    std::fs::write(output_path, file_content)
        .map_err(|e| OpenAIError::FileSaveError(e.to_string()))
}

pub async fn create_batch<C: OpenAIConfigInterface>(
    client: &Client<C>,
    input_file_id: String,
) -> Result<Batch, OpenAIError> {
    let batch_request = BatchRequest {
        input_file_id,
        endpoint: BatchEndpoint::V1ChatCompletions,
        completion_window: BatchCompletionWindow::W24H,
        metadata: None,
    };

    let batch = client.batches().create(batch_request).await?;
    Ok(batch)
}

