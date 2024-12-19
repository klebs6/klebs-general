crate::ix!();

#[derive(Default,Hash,Debug,Clone,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
pub struct OtherLanguage(String);

impl OtherLanguage {

    pub fn new(x: impl ToString) -> Self {
        Self(x.to_string())
    }
}

impl ItemFeature for OtherLanguage {

    fn text(&self) -> Cow<'_,str> {
        Cow::Borrowed(&self.0)
    }
}

impl RandConstruct for OtherLanguage {

    fn random() -> Self {
        Self("UnknownLanguage".to_string())
    }

    fn random_with_rng<R: rand::Rng + ?Sized>(_: &mut R) -> Self {
        Self("UnknownLanguage".to_string())
    }

    fn uniform() -> Self {
        Self::random()
    }
}

