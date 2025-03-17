// ---------------- [ File: workspacer-analysis/src/workspace_analysis.rs ]
crate::ix!();

#[async_trait]
pub trait Analyze {
    type Analysis;
    type Error;

    async fn analyze(&self) -> Result<Self::Analysis, Self::Error>;
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> Analyze for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    type Analysis = WorkspaceSizeAnalysis;
    type Error    = WorkspaceError;

    async fn analyze(&self) -> Result<Self::Analysis, Self::Error> {

        let mut builder = WorkspaceSizeAnalysis::builder();

        for crate_handle in self.crates() {
            let crate_analysis = CrateAnalysis::new(crate_handle).await?;
            builder.add_crate_analysis(crate_analysis);
        }

        Ok(builder.build())
    }
}

#[derive(Debug,Clone)]
pub struct WorkspaceSizeAnalysis {
    crate_analyses: Vec<CrateAnalysis>, // Collection of crate analyses

    // Workspace-level metrics
    total_file_size:        u64,
    total_lines_of_code:    usize,
    total_source_files:     usize,
    total_test_files:       usize,
    largest_file_size:      u64,
    smallest_file_size:     u64,
    average_file_size:      f64,
    average_lines_per_file: f64,
}

impl WorkspaceSizeAnalysis {
    /// Starts the builder for `WorkspaceSizeAnalysis`
    pub fn builder() -> WorkspaceAnalysisBuilder {
        WorkspaceAnalysisBuilder::new()
    }

    // --- Accessors ---
    
    pub fn crate_analyses(&self) -> &Vec<CrateAnalysis> {
        &self.crate_analyses
    }

    pub fn total_file_size(&self) -> u64 {
        self.total_file_size
    }

    pub fn total_lines_of_code(&self) -> usize {
        self.total_lines_of_code
    }

    pub fn total_source_files(&self) -> usize {
        self.total_source_files
    }

    pub fn total_test_files(&self) -> usize {
        self.total_test_files
    }

    pub fn largest_file_size(&self) -> u64 {
        self.largest_file_size
    }

    pub fn smallest_file_size(&self) -> u64 {
        self.smallest_file_size
    }

    pub fn average_file_size(&self) -> f64 {
        self.average_file_size
    }

    pub fn average_lines_per_file(&self) -> f64 {
        self.average_lines_per_file
    }
}

pub struct WorkspaceAnalysisBuilder {
    crate_analyses: Vec<CrateAnalysis>, // Collection of crate analyses
}

impl WorkspaceAnalysisBuilder {
    pub fn new() -> Self {
        Self {
            crate_analyses: Vec::new(),
        }
    }

    /// Adds a crate analysis to the builder
    pub fn add_crate_analysis(&mut self, analysis: CrateAnalysis) -> &mut Self {
        self.crate_analyses.push(analysis);
        self
    }

    /// Builds and returns the `WorkspaceSizeAnalysis` by calculating workspace-level metrics
    pub fn build(&self) -> WorkspaceSizeAnalysis {
        let mut total_file_size     = 0;
        let mut total_lines_of_code = 0;
        let mut total_source_files  = 0;
        let mut total_test_files    = 0;
        let mut largest_file_size   = 0;
        let mut smallest_file_size  = u64::MAX;

        // Aggregate data from each crate analysis
        for crate_analysis in &self.crate_analyses {
            total_file_size     += crate_analysis.total_file_size();
            total_lines_of_code += crate_analysis.total_lines_of_code();
            total_source_files  += crate_analysis.total_source_files();
            total_test_files    += crate_analysis.total_test_files();
            largest_file_size   = largest_file_size.max(crate_analysis.largest_file_size());
            smallest_file_size  = smallest_file_size.min(crate_analysis.smallest_file_size());
        }

        let average_file_size = if total_source_files > 0 {
            total_file_size as f64 / total_source_files as f64
        } else {
            0.0
        };
        let average_lines_per_file = if total_source_files > 0 {
            total_lines_of_code as f64 / total_source_files as f64
        } else {
            0.0
        };

        WorkspaceSizeAnalysis {
            crate_analyses: self.crate_analyses.clone(),
            total_file_size,
            total_lines_of_code,
            total_source_files,
            total_test_files,
            largest_file_size,
            smallest_file_size,
            average_file_size,
            average_lines_per_file,
        }
    }
}

#[cfg(test)]
mod test_workspace_analysis {
    use super::*;

    // -------------------------------------------------------------------------
    // A mock crate handle that we can analyze with CrateAnalysis::new(...)
    // -------------------------------------------------------------------------
    struct MockCrateHandle {
        root_dir:      TempDir,
        has_tests:     bool,
        src_files:     Vec<PathBuf>,
        test_files:    Vec<PathBuf>,
    }

    impl MockCrateHandle {
        fn new() -> Self {
            Self {
                root_dir: tempfile::tempdir().unwrap(),
                has_tests: false,
                src_files: vec![],
                test_files: vec![],
            }
        }

        async fn add_src_file(&mut self, file_name: &str, contents: &str) {
            let src_dir = self.root_dir.path().join("src");
            tokio::fs::create_dir_all(&src_dir).await.unwrap();

            let path = src_dir.join(file_name);
            let mut file = File::create(&path).await.unwrap();
            file.write_all(contents.as_bytes()).await.unwrap();
            // Ensure the file is flushed
            file.sync_all().await.unwrap();
            drop(file);

            self.src_files.push(path);
        }

        async fn add_test_file(&mut self, file_name: &str, contents: &str) {
            let test_dir = self.root_dir.path().join("tests");
            tokio::fs::create_dir_all(&test_dir).await.unwrap();

            let path = test_dir.join(file_name);
            let mut file = File::create(&path).await.unwrap();
            file.write_all(contents.as_bytes()).await.unwrap();
            file.sync_all().await.unwrap();
            drop(file);

            self.test_files.push(path);
            self.has_tests = true;
        }
    }

    #[async_trait]
    impl GetSourceFilesWithExclusions for MockCrateHandle {
        async fn source_files_excluding(&self, _exclude: &[&str]) -> Result<Vec<PathBuf>, CrateError> {
            // For simplicity in this mock, we ignore the exclusion list
            Ok(self.src_files.clone())
        }
    }

    #[async_trait]
    impl GetTestFiles for MockCrateHandle {
        async fn test_files(&self) -> Result<Vec<PathBuf>, CrateError> {
            Ok(self.test_files.clone())
        }
    }

    impl HasTestsDirectory for MockCrateHandle {
        fn has_tests_directory(&self) -> bool {
            self.has_tests
        }
    }

    // -------------------------------------------------------------------------
    // MockWorkspace that contains multiple MockCrateHandles
    // -------------------------------------------------------------------------
    struct MockWorkspace {
        crates: Vec<MockCrateHandle>,
    }

    impl MockWorkspace {
        fn new() -> Self {
            Self { crates: vec![] }
        }

        fn add_crate_handle(&mut self, crate_handle: MockCrateHandle) -> &mut Self {
            self.crates.push(crate_handle);
            self
        }
    }

    // This lets: `for crate_handle in &mock_workspace { ... }`
    impl<'a> IntoIterator for &'a MockWorkspace {
        type Item     = &'a MockCrateHandle;
        type IntoIter = std::slice::Iter<'a, MockCrateHandle>;

        fn into_iter(self) -> Self::IntoIter {
            self.crates.iter()
        }
    }

    // -------------------------------------------------------------------------
    // Implement the Analyze trait for our mock workspace:
    // this matches the real logic in your code.
    // -------------------------------------------------------------------------
    #[async_trait]
    impl Analyze for MockWorkspace {
        type Analysis = WorkspaceSizeAnalysis;
        type Error    = WorkspaceError;

        async fn analyze(&self) -> Result<Self::Analysis, Self::Error> {
            let mut builder = WorkspaceSizeAnalysis::builder();

            for crate_handle in self {
                let crate_analysis = CrateAnalysis::new(crate_handle).await?;
                builder.add_crate_analysis(crate_analysis);
            }

            Ok(builder.build())
        }
    }

    // -------------------------------------------------------------------------
    // Now we can test workspace-level analysis with multiple crates.
    // -------------------------------------------------------------------------

    // 1) No crates at all
    #[tokio::test]
    async fn test_no_crates_in_workspace() {
        let workspace = MockWorkspace::new();
        let analysis = workspace.analyze().await.unwrap();

        assert_eq!(analysis.crate_analyses().len(), 0);
        assert_eq!(analysis.total_file_size(), 0);
        assert_eq!(analysis.total_lines_of_code(), 0);
        assert_eq!(analysis.total_source_files(), 0);
        assert_eq!(analysis.total_test_files(), 0);
        assert_eq!(analysis.largest_file_size(), 0);
        assert_eq!(analysis.smallest_file_size(), u64::MAX);
        assert_eq!(analysis.average_file_size(), 0.0);
        assert_eq!(analysis.average_lines_per_file(), 0.0);
    }

    // 2) Single crate with no files
    #[tokio::test]
    async fn test_single_crate_no_files() {
        let mut workspace = MockWorkspace::new();
        let crate_empty = MockCrateHandle::new(); // No src or test files

        workspace.add_crate_handle(crate_empty);
        let analysis = workspace.analyze().await.unwrap();

        assert_eq!(analysis.crate_analyses().len(), 1);
        assert_eq!(analysis.total_file_size(), 0);
        assert_eq!(analysis.total_lines_of_code(), 0);
        assert_eq!(analysis.total_source_files(), 0);
        assert_eq!(analysis.total_test_files(), 0);
        assert_eq!(analysis.largest_file_size(), 0);
        assert_eq!(analysis.smallest_file_size(), u64::MAX);
        // Because total_source_files is 0, average is 0.0
        assert_eq!(analysis.average_file_size(), 0.0);
        assert_eq!(analysis.average_lines_per_file(), 0.0);
    }

    // 3) Single crate with multiple files (src and tests)
    #[tokio::test]
    async fn test_single_crate_multiple_files() {
        let mut crate_handle = MockCrateHandle::new();
        let src_a = "fn main() {}\nprintln!(\"hello\");";
        let src_b = "mod sub;\nmod sub2;";
        let test_a = "test fn1\ntest fn2\ntest fn3";

        crate_handle.add_src_file("main.rs", src_a).await;
        crate_handle.add_src_file("lib.rs", src_b).await;
        crate_handle.add_test_file("test_something.rs", test_a).await;

        // Put into workspace
        let mut workspace = MockWorkspace::new();
        workspace.add_crate_handle(crate_handle);

        // Analyze
        let analysis = workspace.analyze().await.unwrap();
        assert_eq!(analysis.crate_analyses().len(), 1);

        // Summaries
        // src_a has 2 lines, src_b has 2, test_a has 3 => 7 total
        assert_eq!(analysis.total_lines_of_code(), 7);
        // total source files = 2, total test files = 1
        assert_eq!(analysis.total_source_files(), 2);
        assert_eq!(analysis.total_test_files(), 1);

        // File sizes
        let size_a = src_a.len() as u64;
        let size_b = src_b.len() as u64;
        let size_test = test_a.len() as u64;
        let total_size = size_a + size_b + size_test;
        assert_eq!(analysis.total_file_size(), total_size);
        assert_eq!(analysis.largest_file_size(), size_a.max(size_b).max(size_test));
        assert_eq!(analysis.smallest_file_size(), size_a.min(size_b).min(size_test));

        // Averages are computed over the total source files (not including test files).
        // So we have 2 source files => average_file_size and average_lines_per_file
        let avg_file_size = total_size as f64 / 2.0;
        let avg_lines     = 7.0 / 2.0;
        // floating comparisons
        assert!((analysis.average_file_size() - avg_file_size).abs() < f64::EPSILON);
        assert!((analysis.average_lines_per_file() - avg_lines).abs() < f64::EPSILON);
    }

    // 4) Multiple crates in the same workspace
    #[tokio::test]
    async fn test_multiple_crates() {
        // First crate
        let mut crate1 = MockCrateHandle::new();
        crate1.add_src_file("file1.rs", "line1\nline2").await; // 2 lines
        crate1.add_test_file("test1.rs", "test\nstuff").await; // 2 lines

        // Second crate
        let mut crate2 = MockCrateHandle::new();
        // no test files
        crate2.add_src_file("file2.rs", "foo\nbar\nbaz").await; // 3 lines
        crate2.add_src_file("file3.rs", "").await;              // 0 lines

        // Third crate
        let crate3 = MockCrateHandle::new(); // empty

        // Build the workspace
        let mut workspace = MockWorkspace::new();
        workspace
            .add_crate_handle(crate1)
            .add_crate_handle(crate2)
            .add_crate_handle(crate3);

        // Analyze
        let analysis = workspace.analyze().await.unwrap();
        assert_eq!(analysis.crate_analyses().len(), 3);

        // Summations across all crates
        // lines: crate1 => 2+2=4, crate2 => 3+0=3, crate3 => 0 => total=7
        assert_eq!(analysis.total_lines_of_code(), 7);

        // source files: crate1 => 1, crate2 => 2, crate3 => 0 => total=3
        assert_eq!(analysis.total_source_files(), 3);

        // test files: crate1 => 1, crate2 => 0, crate3 => 0 => total=1
        assert_eq!(analysis.total_test_files(), 1);

        // file sizes
        let size_c1_f1 = "line1\nline2".len() as u64; // 11
        let size_c1_t1 = "test\nstuff".len() as u64;  // 10
        let size_c2_f1 = "foo\nbar\nbaz".len() as u64; // 11
        let size_c2_f2 = "".len() as u64;              // 0
        // crate3 => no files => size=0
        let total_file_size = size_c1_f1 + size_c1_t1 + size_c2_f1 + size_c2_f2;
        assert_eq!(analysis.total_file_size(), total_file_size);

        let largest_file_size = *[
            size_c1_f1,
            size_c1_t1,
            size_c2_f1,
            size_c2_f2,
            0 // crate3
        ]
        .iter()
        .max()
        .unwrap();
        let smallest_file_size = *[
            size_c1_f1,
            size_c1_t1,
            size_c2_f1,
            size_c2_f2,
            u64::MAX // if none, that is the default, but we do have files
        ]
        .iter()
        .min()
        .unwrap();
        assert_eq!(analysis.largest_file_size(), largest_file_size);
        assert_eq!(analysis.smallest_file_size(), smallest_file_size);

        // average_file_size and average_lines_per_file are per "source file" (3 total).
        // total_file_size = 11 + 10 + 11 + 0 = 32
        // total_source_files = 3
        // => average_file_size = 32 / 3
        let expected_avg_size = 32.0 / 3.0;
        // total_lines_of_code=7 => 7/3=2.3333...
        let expected_avg_lines = 7.0 / 3.0;
        assert!((analysis.average_file_size() - expected_avg_size).abs() < f64::EPSILON);
        assert!((analysis.average_lines_per_file() - expected_avg_lines).abs() < f64::EPSILON);
    }

    // 5) Confirm average is zero if no source files exist
    #[tokio::test]
    async fn test_all_test_files_no_source_files() {
        let mut crate_with_tests = MockCrateHandle::new();
        crate_with_tests.add_test_file("test_only.rs", "one\ntwo\nthree").await; // 3 lines

        let mut workspace = MockWorkspace::new();
        workspace.add_crate_handle(crate_with_tests);

        let analysis = workspace.analyze().await.unwrap();

        // 0 source files => 1 test file
        assert_eq!(analysis.total_source_files(), 0);
        assert_eq!(analysis.total_test_files(), 1);
        // lines => 3
        assert_eq!(analysis.total_lines_of_code(), 3);
        // Because total_source_files is 0, average_file_size & average_lines_per_file = 0.0
        assert_eq!(analysis.average_file_size(), 0.0);
        assert_eq!(analysis.average_lines_per_file(), 0.0);
    }
}
