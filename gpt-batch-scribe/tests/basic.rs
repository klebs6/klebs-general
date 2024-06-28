use gpt_batch_scribe::*;
use traced_test::*;
use tracing_setup::*;

#[traced_test]
fn basic_functionality() -> Result<(),GptBatchCreationError> {

    let request = GptBatchAPIRequest::new_with_image(
        0, 
        "this is the system message", 
        "this is the user message", 
        "this is the image b64 content"
    );

    let json = serde_json::to_string_pretty(&request)?;

    info!("{}", json);

    assert!(false); // this triggers the tracing

    Ok(())
}
