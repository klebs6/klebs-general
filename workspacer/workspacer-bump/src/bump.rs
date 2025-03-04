// ---------------- [ File: src/bump.rs ]
crate::ix!();

// ---------------------- [ File: workspace-bump/src/lib.rs ] ----------------------

/// A trait describing how to bump versions in a workspace.
#[async_trait]
pub trait BumpAll {
    type Error;
    /// Bump all crates in the workspace by the specified release type.
    async fn bump_all(&mut self, release: ReleaseType) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait BumpCrateAndDownstreams {
    type Error;

    /// Bump a single crate and any crates that depend on it. Then rewrite
    /// their Cargo.toml references so they point at the newly bumped version.
    async fn bump_crate_and_downstreams(
        &mut self,
        crate_name: &mut CrateHandle,
        release: ReleaseType
    ) -> Result<(), Self::Error>;
}

/// This will be implemented by CrateHandle
#[async_trait]
pub trait Bump {
    type Error;

    async fn bump(
        &mut self,
        release: ReleaseType
    ) -> Result<(), Self::Error>;
}
