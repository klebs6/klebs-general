// ---------------- [ File: workspacer-cratesio-mock/src/publish.rs ]
crate::ix!();

#[post("/api/v1/crates/new", data = "<upload>")]
pub async fn publish_new(
    upload: Data<'_>,
    state: &rocket::State<AppState>,
) -> (Status, Json<serde_json::Value>) {
    trace!("Entering publish_new endpoint.");

    // We'll read up to (limit + 1) bytes. If we read more than `limit`, 
    // we know the request exceeded the intended max size.
    let limit = 10u64.mebibytes();
    let limit_plus_one = limit + 1;
    let mut buf = Vec::new();

    match upload.open(limit_plus_one).read_to_end(&mut buf).await {
        Ok(bytes_read) => {
            debug!("Read {} bytes from incoming request body.", bytes_read);

            // If we actually read more bytes than `limit`, it's truncated.
            if bytes_read > limit {
                warn!(
                    "Request exceeded the {}-byte limit; read was truncated.",
                    limit
                );
                let err = PublishErrResponseBuilder::default()
                    .errors(vec![
                        PublishErrObjectBuilder::default()
                            .detail("Failed reading request body".to_string())
                            .build()
                            .unwrap()
                    ])
                    .build()
                    .unwrap();
                return (Status::BadRequest, Json(serde_json::json!(err)));
            }

            let text = String::from_utf8_lossy(&buf);
            match serde_json::from_str::<StoredCrate>(&text) {
                Ok(parsed) => {
                    info!("Parsed crate: name={}, vers={}", parsed.name(), parsed.vers());
                    if parsed.description().is_none() {
                        warn!("Description field is missing or empty in request.");
                        let err = PublishErrResponseBuilder::default()
                            .errors(vec![
                                PublishErrObjectBuilder::default()
                                    .detail("missing or empty metadata fields: description".to_string())
                                    .build()
                                    .unwrap()
                            ])
                            .build()
                            .unwrap();
                        return (Status::BadRequest, Json(serde_json::json!(err)));
                    }

                    let mut db = state.db().lock().await;
                    let crate_entry = db
                        .published_mut()
                        .entry(parsed.name().clone())
                        .or_insert_with(HashMap::new);

                    if crate_entry.contains_key(parsed.vers().as_str()) {
                        warn!("Crate version already exists in the in-memory DB.");
                        let err = PublishErrResponseBuilder::default()
                            .errors(vec![
                                PublishErrObjectBuilder::default()
                                    .detail(format!("crate {} already exists", parsed.name()))
                                    .build()
                                    .unwrap()
                            ])
                            .build()
                            .unwrap();
                        return (Status::BadRequest, Json(serde_json::json!(err)));
                    }

                    crate_entry.insert(parsed.vers().clone(), parsed.clone());
                    info!("Successfully stored crate {}@{}", parsed.name(), parsed.vers());

                    let resp = PublishOkResponseBuilder::default()
                        .ok(true)
                        .build()
                        .unwrap();
                    (Status::Ok, Json(serde_json::json!(resp)))
                }
                Err(e) => {
                    warn!("Failed to parse JSON: {}", e);
                    let err = PublishErrResponseBuilder::default()
                        .errors(vec![
                            PublishErrObjectBuilder::default()
                                .detail(format!("invalid JSON input: {}", e))
                                .build()
                                .unwrap()
                        ])
                        .build()
                        .unwrap();
                    (Status::BadRequest, Json(serde_json::json!(err)))
                }
            }
        }
        Err(e) => {
            error!("Error reading request body: {:?}", e);
            let err = PublishErrResponseBuilder::default()
                .errors(vec![
                    PublishErrObjectBuilder::default()
                        .detail("Failed reading request body".to_string())
                        .build()
                        .unwrap()
                ])
                .build()
                .unwrap();
            (Status::BadRequest, Json(serde_json::json!(err)))
        }
    }
}
