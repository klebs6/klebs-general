// ---------------- [ File: src/load_batch_files.rs ]
crate::ix!();

pub async fn load_input_file(path: impl AsRef<Path>) 
    -> Result<BatchInputData, JsonParseError> 
{
    info!("loading input file: {:?}", path.as_ref());

    let file      = File::open(path).await?;
    let reader    = BufReader::new(file);
    let mut lines = reader.lines();

    let mut requests = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let request: LanguageModelBatchAPIRequest = serde_json::from_str(&line)?;
        requests.push(request);
    }

    Ok(BatchInputData::new(requests))
}

pub async fn load_error_file(path: impl AsRef<Path>) 
    -> Result<BatchErrorData, JsonParseError> 
{
    info!("loading error file: {:?}", path.as_ref());

    let file      = File::open(path).await?;
    let reader    = BufReader::new(file);
    let mut lines = reader.lines();

    let mut responses = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let response_record: BatchResponseRecord = serde_json::from_str(&line)?;
        responses.push(response_record);
    }

    Ok(BatchErrorData::new(responses))
}

pub async fn load_output_file(path: impl AsRef<Path>) 
    -> Result<BatchOutputData, JsonParseError> 
{
    info!("loading output file: {:?}", path.as_ref());

    let file      = File::open(path).await?;
    let reader    = BufReader::new(file);
    let mut lines = reader.lines();

    let mut responses = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let response_record: BatchResponseRecord = serde_json::from_str(&line)?;
        responses.push(response_record);
    }

    Ok(BatchOutputData::new(responses))
}
