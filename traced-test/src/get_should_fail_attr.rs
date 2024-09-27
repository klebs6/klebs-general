crate::ix!();

pub trait HasShouldFailAttr {

    fn should_fail_attr(&self) -> Option<ShouldFailAttr>;
}

pub trait MaybeHasExpectedFailureMessage {

    fn expected_failure_message(&self)
        -> Option<Cow<'_,str>>;
}

pub trait CheckForAndRetrieveTheUniqueShouldFailAttr {

    fn maybe_get_should_fail_attr(&self)
        -> Result<Option<ShouldFailAttr>, TracedTestError>;
}

impl CheckForAndRetrieveTheUniqueShouldFailAttr for &[syn::Attribute] {

    fn maybe_get_should_fail_attr(&self) -> Result<Option<ShouldFailAttr>, TracedTestError> {
        let mut should_fail_attr = None;

        for attr in *self {
            if attr.path().is_ident("should_fail") {
                if should_fail_attr.is_some() {
                    return Err(TracedTestError::MultipleShouldFailAttrs);
                }

                let parsed_attr =
                    ShouldFailAttr::try_from(attr.clone()).map_err(TracedTestError::ShouldFailAttrError)?;
                should_fail_attr = Some(parsed_attr);
            }
        }

        Ok(should_fail_attr)
    }
}
