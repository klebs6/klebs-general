// ---------------- [ File: language-model-token-expander/src/token_packaged.rs ]
crate::ix!();

#[derive(Clone,Debug)]
pub struct TokenPackagedForExpansion(CamelCaseTokenWithComment);

impl From<&CamelCaseTokenWithComment> for TokenPackagedForExpansion {
    fn from(x: &CamelCaseTokenWithComment) -> Self {
        Self(x.clone())
    }
}

impl std::fmt::Display for TokenPackagedForExpansion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0)
    }
}

unsafe impl Send for TokenPackagedForExpansion {}
unsafe impl Sync for TokenPackagedForExpansion {}

impl IntoLanguageModelQueryString for TokenPackagedForExpansion {
    fn into_language_model_query_string(&self) -> String {
        self.0.clone().into()
    }
}

impl Named for TokenPackagedForExpansion {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        self.0.name()
    }
}

impl HasAssociatedOutputName for TokenPackagedForExpansion
{
    fn associated_output_name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Owned(format!("{}-expansion", self.0.name()))
    }
}
