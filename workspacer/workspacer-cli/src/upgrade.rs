crate::ix!();

#[derive(Debug, StructOpt)]
pub enum UpgradeSubcommand {
    /// Upgrade function-level tracing
    FunctionTracing {
        #[structopt(long = "crate")]
        crate_name: Option<String>,

        #[structopt(long = "file")]
        file_name: Option<String>,

        #[structopt(long = "fn")]
        function_name: Option<String>,
    },
    /// Upgrade a single test with new patterns
    Test {
        #[structopt(long = "crate")]
        crate_name: Option<String>,

        #[structopt(long = "test-name")]
        test_name: Option<String>,
    },
    /// Upgrade multiple test suites
    TestSuites {
        #[structopt(long = "crate")]
        crate_name: Option<String>,

        #[structopt(long = "file")]
        file_name: Option<String>,

        #[structopt(long = "fn")]
        function_name: Option<String>,

        #[structopt(long = "suite-name")]
        suite_name: Option<String>,
    },
    /// Upgrade the tracing in a single test suite
    TestSuiteTracing {
        #[structopt(long = "crate")]
        crate_name: Option<String>,

        #[structopt(long = "suite-name")]
        suite_name: Option<String>,

        #[structopt(long = "file")]
        file_name: Option<String>,
    },
    /// Upgrade the tracing in a single test
    TestTracing {
        #[structopt(long = "crate")]
        crate_name: Option<String>,

        #[structopt(long = "test-name")]
        test_name: Option<String>,
    },

    /// Upgrade overall tracing approach
    Tracing {
        #[structopt(long = "path")]
        workspace_path: Option<String>,
    },
}

impl UpgradeSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
