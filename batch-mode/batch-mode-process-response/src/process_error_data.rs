// ---------------- [ File: batch-mode-process-response/src/process_error_data.rs ]
crate::ix!();

pub async fn process_error_data(error_data: &BatchErrorData) 
-> Result<(), BatchErrorProcessingError> 
{
    for response_record in error_data.responses() {
        if let Some(error_body) = response_record.response().body().as_error() {
            eprintln!(
                "Error for Custom ID '{}': Code: {:?}, Message: {}",
                response_record.custom_id(),
                error_body.error().code(),
                error_body.error().message()
            );
            // Handle the error as needed
        }
    }
    Ok(())
}

#[cfg(test)]
mod process_error_data_tests {
    use super::*;

    #[traced_test]
    async fn test_process_error_data() {
        // Supply error_type
        let err_details = BatchErrorDetailsBuilder::default()
            .error_type(ErrorType::Unknown("test_err".to_string()))
            .code(Some("401".to_string()))
            .message("Unauthorized".to_string())
            .build()
            .unwrap();

        let err_body = BatchErrorResponseBodyBuilder::default()
            .error(err_details)
            .build()
            .unwrap();

        let response_content = BatchResponseContentBuilder::default()
            .status_code(400_u16)
            .request_id(ResponseRequestId::new("resp_err1"))
            .body(BatchResponseBody::Error(err_body))
            .build()
            .unwrap();

        let record = BatchResponseRecordBuilder::default()
            .id(BatchRequestId::new("id"))
            .custom_id(CustomRequestId::new("err1"))
            .response(response_content)
            .build()
            .unwrap();

        let error_data = BatchErrorDataBuilder::default()
            .responses(vec![record])
            .build()
            .unwrap();

        let result = process_error_data(&error_data).await;
        assert!(result.is_ok());
    }
}
