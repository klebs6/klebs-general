// ---------------- [ File: workspacer-cratesio-mock/src/app_state.rs ]
crate::ix!();

#[derive(Getters, Setters, Builder, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct AppState {
    #[builder(default)]
    db: Arc<AsyncMutex<MockCratesDb>>,
}
