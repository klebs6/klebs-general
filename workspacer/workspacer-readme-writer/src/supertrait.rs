// ---------------- [ File: workspacer-readme-writer/src/supertrait.rs ]
crate::ix!();

pub trait ReadmeWritingCrateHandle<P>:
    CrateHandleInterface<P>
    + ApplyAiReadmeOutput<Error = CrateError>
    + ConsolidateCrateInterface
    + Send
    + Sync
    + Debug
    + 'static
    where
        P: AsRef<Path> + Send + Sync + 'static,
{
    // no extra methods needed, just a trait bundling
}

impl<P, T> ReadmeWritingCrateHandle<P> for T
where
    T: CrateHandleInterface<P>
        + ApplyAiReadmeOutput<Error = CrateError>
        + ConsolidateCrateInterface
        + Send
        + Sync
        + Debug
        + 'static,
    P: AsRef<Path> + Send + Sync + 'static,
{
    // blanket impl, so any T satisfying these constraints
    // automatically implements ReadmeWritingCrateHandle<P>.
}
