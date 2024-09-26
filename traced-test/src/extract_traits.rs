crate::ix!();

pub trait CheckForAndRetrieveTheUniqueShouldFailAttr {

    fn maybe_get_should_fail_attr(&self)
        -> Result<Option<ShouldFailAttr>, TracedTestError>;
}

pub trait MaybeHasExpectedFailureMessage {

    fn expected_failure_message(&self)
        -> Option<Cow<'_,str>>;
}

pub trait MaybeHasPanicMessage {

    fn panic_message(&self)
        -> Option<Cow<'_, str>>;
}

pub trait CheckForAndRetrieveTheUniqueShouldPanicAttr {

    /// Returns `Some(ShouldPanicAttr)` if the attribute is found.
    /// Returns `Err` if there's a duplicate or any other error while parsing.
    fn maybe_get_should_panic_attr(&self)
        -> Result<Option<ShouldPanicAttr>, ShouldPanicAttrError>;
}
