// ---------------- [ File: workspacer-cratesio-mock/src/protocol.rs ]
crate::ix!();

#[derive(Clone,Getters, Setters, Builder, Debug, Serialize, Default)]
#[getset(get = "pub", set = "pub")]
pub struct PublishOkResponse {
    #[builder(default)]
    ok: bool,
}

#[derive(Clone,Getters, Setters, Builder, Debug, Serialize, Default)]
#[getset(get = "pub", set = "pub")]
pub struct PublishErrResponse {
    #[builder(default)]
    errors: Vec<PublishErrObject>,
}

#[derive(Clone,Getters, Setters, Builder, Debug, Serialize, Default)]
#[getset(get = "pub", set = "pub")]
pub struct PublishErrObject {
    #[builder(default)]
    detail: String,
}

// For 404 etc.
#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "errors": [
            {
                "detail": format!("No route for {}", req.uri())
            }
        ]
    }))
}
