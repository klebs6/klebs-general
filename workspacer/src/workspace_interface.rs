// ---------------- [ File: src/workspace_interface.rs ]
crate::ix!();

pub trait WorkspaceInterface<P>
: GetCrates
+ NumCrates
+ CleanupWorkspace
+ WatchAndReload
+ RunTestsWithCoverage
+ GetCargoMetadata
+ RebuildOrTest
+ Analyze
+ GenerateDocs
+ RunLinting
+ DetectCircularDependencies
+ GenerateDependencyTree
+ ValidateIntegrity
+ ReadyForCargoPublish
+ AsyncTryFrom<P>
+ AsyncIsValid
+ AsyncFindItems
+ AsRef<Path>
where 
for<'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait
{}


pub trait GetCrates {
    fn crates(&self) -> &[CrateHandle];
}

pub trait NumCrates {
    fn n_crates(&self) -> usize;
}

#[async_trait]
pub trait CleanupWorkspace {

    async fn cleanup_workspace(&self) -> Result<(), WorkspaceError>;
}

#[async_trait]
pub trait WatchAndReload {

    type Error;

    async fn watch_and_reload(
        &self,
        tx: Option<mpsc::Sender<Result<(), WorkspaceError>>>,
        runner: Arc<dyn CommandRunner + Send + Sync + 'static>,
        cancel_token: CancellationToken,
    ) -> Result<(), Self::Error>;

    fn is_relevant_change(&self, path: &Path) -> bool;
}

#[async_trait]
pub trait RunTestsWithCoverage {

    type Report;
    type Error;

    async fn run_tests_with_coverage(&self) 
        -> Result<Self::Report, Self::Error>;
}

#[async_trait]
pub trait GetCargoMetadata {

    type Error;
    async fn get_cargo_metadata(&self) -> Result<Metadata, Self::Error>;
}

#[async_trait]
pub trait RebuildOrTest {

    type Error;

    async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait Analyze {
    type Analysis;
    type Error;

    async fn analyze(&self) -> Result<Self::Analysis, Self::Error>;
}

#[async_trait]
pub trait GenerateDocs {
    type Error;
    async fn generate_docs(&self) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait RunLinting {

    type Report;
    type Error;
    async fn run_linting(&self) -> Result<Self::Report, Self::Error>;
}

#[async_trait]
pub trait DetectCircularDependencies {

    type Error;
    async fn detect_circular_dependencies(&self) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait GenerateDependencyTree {

    type Tree;
    type Error;

    async fn generate_dependency_tree(&self) -> Result<Self::Tree, Self::Error>;
    async fn generate_dependency_tree_dot(&self) -> Result<String, Self::Error>;
}
