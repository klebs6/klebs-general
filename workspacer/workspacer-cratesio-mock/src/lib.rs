// ---------------- [ File: workspacer-cratesio-mock/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{app_state}
x!{crates_db}
x!{protocol}
x!{publish}
x!{stored_crate}

#[cfg(test)]
mod test_publish_integration {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use rocket::serde::json::serde_json;
    use tracing::{trace, debug, info, warn, error};
    use crate::app_state::AppStateBuilder;
    use crate::crates_db::MockCratesDb;
    use crate::stored_crate::StoredCrate;
    use getset::*;
    use derive_builder::*;

    // We stand up a fresh Rocket for each test to ensure 
    // we're truly testing the interface in isolation.
    fn setup_rocket() -> Client {
        trace!("Setting up test rocket instance.");

        let state = AppStateBuilder::default()
            .db(Arc::new(AsyncMutex::new(MockCratesDb::default())))
            .build()
            .unwrap();

        let rocket = rocket::build()
            .manage(state)
            .mount("/", rocket::routes![publish_new])
            .register("/", rocket::catchers![not_found]);

        Client::tracked(rocket).expect("valid rocket instance")
    }

    #[traced_test]
    fn test_successful_publish() {
        trace!("Starting test_successful_publish.");

        let client = setup_rocket();
        let body = serde_json::json!({
            "name": "my-crate",
            "vers": "0.1.0",
            "description": "A test crate."
        })
        .to_string();

        debug!("POST body: {}", body);

        let response = client
            .post("/api/v1/crates/new")
            .header(ContentType::JSON)
            .body(body)
            .dispatch();

        info!("Response status: {:?}", response.status());
        assert_eq!(response.status(), Status::Ok);

        let response_json = response.into_json::<serde_json::Value>()
            .expect("Response should be valid JSON");
        debug!("Response JSON: {:?}", response_json);

        // Expecting structure like: {"ok":true}
        assert!(response_json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));
    }

    #[traced_test]
    fn test_missing_description() {
        trace!("Starting test_missing_description.");

        let client = setup_rocket();
        let body = serde_json::json!({
            "name": "my-crate",
            "vers": "0.2.0"
            // no "description"
        })
        .to_string();

        debug!("POST body: {}", body);

        let response = client
            .post("/api/v1/crates/new")
            .header(ContentType::JSON)
            .body(body)
            .dispatch();

        info!("Response status: {:?}", response.status());
        assert_eq!(response.status(), Status::BadRequest);

        let response_json = response.into_json::<serde_json::Value>()
            .expect("Response should be valid JSON");
        debug!("Response JSON: {:?}", response_json);

        // Expecting structure like: {"errors":[{"detail":"..."}]}
        let errors = response_json
            .get("errors")
            .and_then(|v| v.as_array())
            .expect("Expected 'errors' array in response");
        let detail = errors[0]
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            detail.contains("description"),
            "Should complain about missing description"
        );
    }

    #[traced_test]
    fn test_already_published() {
        trace!("Starting test_already_published.");

        let client = setup_rocket();

        // First publish
        let body = serde_json::json!({
            "name": "my-crate",
            "vers": "1.0.0",
            "description": "Initial publish."
        })
        .to_string();

        let first_resp = client
            .post("/api/v1/crates/new")
            .header(ContentType::JSON)
            .body(body.clone())
            .dispatch();
        assert_eq!(first_resp.status(), Status::Ok);

        // Second publish of the same exact version
        let second_resp = client
            .post("/api/v1/crates/new")
            .header(ContentType::JSON)
            .body(body)
            .dispatch();

        info!("Response status: {:?}", second_resp.status());
        assert_eq!(second_resp.status(), Status::BadRequest);

        let response_json = second_resp.into_json::<serde_json::Value>()
            .expect("Response should be valid JSON");
        debug!("Response JSON: {:?}", response_json);

        let errors = response_json
            .get("errors")
            .and_then(|v| v.as_array())
            .expect("Expected 'errors' array in response");
        let detail = errors[0]
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            detail.contains("already exists"),
            "Should complain about crate already existing"
        );
    }

    #[traced_test]
    fn test_invalid_json_body() {
        trace!("Starting test_invalid_json_body.");

        let client = setup_rocket();

        // Provide invalid JSON
        let body = "{ invalid-json".to_string();
        debug!("POST body: {}", body);

        let response = client
            .post("/api/v1/crates/new")
            .header(ContentType::JSON)
            .body(body)
            .dispatch();

        info!("Response status: {:?}", response.status());
        assert_eq!(response.status(), Status::BadRequest);

        let response_json = response.into_json::<serde_json::Value>()
            .expect("Response should be valid JSON");
        debug!("Response JSON: {:?}", response_json);

        let errors = response_json
            .get("errors")
            .and_then(|v| v.as_array())
            .expect("Expected 'errors' array in response");
        let detail = errors[0]
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            detail.contains("invalid JSON"),
            "Should complain about invalid JSON"
        );
    }

    #[traced_test]
    fn test_404_catcher() {
        trace!("Starting test_404_catcher.");

        let client = setup_rocket();
        let response = client.get("/this/does/not/exist").dispatch();
        info!("Response status: {:?}", response.status());
        assert_eq!(response.status(), Status::NotFound);

        let response_json = response.into_json::<serde_json::Value>()
            .expect("Response should be valid JSON");
        debug!("Response JSON: {:?}", response_json);

        let errors = response_json
            .get("errors")
            .and_then(|v| v.as_array())
            .expect("Expected 'errors' array in response");
        let detail = errors[0]
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            detail.contains("No route for /this/does/not/exist"),
            "Should contain a 404 message with requested URI"
        );
    }

    #[traced_test]
    fn test_too_large_body() {
        trace!("Starting test_too_large_body.");

        // We'll craft a body exceeding 10MiB (~ 10,485,760 bytes).
        // Let's do 11MiB to be safe. We'll expect a BadRequest 
        // from rocket's 'read_to_end' limit in publish_new.
        let size = 11 * 1024 * 1024; 
        let oversized_body = "A".repeat(size);

        let client = setup_rocket();
        let response = client
            .post("/api/v1/crates/new")
            .header(ContentType::JSON)
            .body(oversized_body)
            .dispatch();

        info!("Response status: {:?}", response.status());
        // Typically rocket yields 413 (Payload Too Large) or 400 
        // depending on the version and data limit handling.
        // In our code, we handle any read error as BadRequest.
        assert_eq!(response.status(), Status::BadRequest);

        let response_json = response.into_json::<serde_json::Value>()
            .expect("Response should be valid JSON");
        debug!("Response JSON: {:?}", response_json);

        let errors = response_json
            .get("errors")
            .and_then(|v| v.as_array())
            .expect("Expected 'errors' array in response");
        let detail = errors[0]
            .get("detail")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            detail.contains("Failed reading request body"),
            "Should mention failed reading body for large input"
        );
    }
}
