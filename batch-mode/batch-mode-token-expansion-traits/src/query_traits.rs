// ---------------- [ File: batch-mode-token-expansion-traits/src/query_traits.rs ]
crate::ix!();

pub trait IntoLanguageModelQueryString {
    fn into_language_model_query_string(&self) -> String;
}

impl IntoLanguageModelQueryString for CamelCaseTokenWithComment {
    fn into_language_model_query_string(&self) -> String {
        self.clone().into()
    }
}
