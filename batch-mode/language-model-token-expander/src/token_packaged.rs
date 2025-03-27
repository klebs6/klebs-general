crate::ix!();

#[derive(Clone,Debug)]
pub struct TokenPackagedForExpansion(CamelCaseTokenWithComment);

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
