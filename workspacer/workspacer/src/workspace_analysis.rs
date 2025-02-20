// ---------------- [ File: src/workspace_analysis.rs ]
crate::ix!();

#[async_trait]
impl<P,H:CrateHandleInterface<P>> Analyze for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

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
