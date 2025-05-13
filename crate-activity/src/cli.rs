crate::ix!();

/// Crate Activity Analyzer
#[derive(Getters,StructOpt, Debug)]
#[structopt(name = "act")]
pub struct CrateActivityCli {

    #[structopt(long, short = "i", help = "Ignores crate activity cache, scrapes activity data again")]
    #[getset(get = "pub")]
    ignore_cache: bool,

    /// Enable all analyses: correlations, PCA, hierarchical clustering, network analysis, etc.
    #[structopt(long, help = "Enable all analyses at once")]
    #[getset(get = "pub")]
    all: bool,

    /// Toggle to enable or disable correlation analysis
    #[structopt(long, short = "c", help = "Display correlation analysis")]
    #[getset(get = "pub")]
    show_correlations: bool,

    /// Toggle to enable or disable PCA analysis
    #[structopt(long, short = "p", help = "Perform PCA analysis")]
    #[getset(get = "pub")]
    perform_pca: bool,

    /// Toggle to enable hierarchical clustering
    #[structopt(long, short = "h", help = "Perform hierarchical clustering")]
    #[getset(get = "pub")]
    perform_hierarchical_clustering: bool,

    /// Toggle to enable correlation network analysis
    #[structopt(long, short = "n", help = "Perform correlation network analysis")]
    #[getset(get = "pub")]
    correlation_network: bool,

    /// Threshold for including edges in the correlation network graph
    #[structopt(long, default_value = "0.7", help = "Correlation threshold for network edges")]
    #[getset(get = "pub")]
    network_threshold: f64,

    /// Use Girvan–Newman algorithm to find a specified number of communities
    #[structopt(long, short = "g", help = "Target number of communities for Girvan–Newman")]
    #[getset(get = "pub")]
    girvan_newman: Option<usize>,

    /// Compute betweenness centrality for nodes (and edges) and display top nodes
    #[structopt(long, short = "b", help = "Compute betweenness centrality and display top nodes")]
    #[getset(get = "pub")]
    compute_betweenness: bool,

    /// Print a summary of the network graph
    #[structopt(long, short = "s", help = "Print a summary of the network graph")]
    #[getset(get = "pub")]
    print_summary: bool,

    /// Toggle to enable time-lagged correlation analysis
    #[structopt(long, short = "t", help = "Compute time-lagged correlations")]
    #[getset(get = "pub")]
    time_lag_correlations: bool,

    /// Maximum lag in days for time-lagged correlations
    #[structopt(long, default_value = "7", help = "Maximum lag (in days) to consider for time-lag correlations")]
    #[getset(get = "pub")]
    max_lag: i32,

    /// Z-score threshold for outlier detection (MAD-based)
    #[structopt(long, default_value = "24.0", help = "Z-score threshold for outlier detection")]
    #[getset(get = "pub")]
    outlier_z_threshold: f64,

    /// Downweight outliers instead of removing them
    #[structopt(long, help = "Downweight outliers instead of removing them")]
    #[getset(get = "pub")]
    downweight_outliers: bool,

    /// Factor by which to downweight outliers if --downweight-outliers is used
    #[structopt(long, default_value = "0.1", help = "Downweight factor for outliers")]
    #[getset(get = "pub")]
    outlier_weight: f64,

    /// Disable outlier handling altogether
    #[structopt(long, help = "Disable outlier detection and handling")]
    disable_outlier_handling: bool,

    #[structopt(long, help = "If true, we will print each individual crate per group")]
    #[getset(get="pub")]
    expand_groups: bool,

    #[structopt(long, default_value = "2", help = "Minimum group size required to treat them as a group")]
    #[getset(get="pub")]
    min_group_size: usize,
}

impl CrateActivityCli {

    pub fn read_command_line() -> Self {
        let mut cli = CrateActivityCli::from_args();
        cli.apply_all_flag();
        cli
    }

    pub fn disable_outlier_handling(&self) -> bool {

        #[cfg(test)]
        let disable_outliers_override = true; // Force no outliers in test

        #[cfg(not(test))]
        let disable_outliers_override = false;

        let disable_outliers = self.disable_outlier_handling || disable_outliers_override;

        disable_outliers
    }

    /// Apply the `--all` flag overrides if set.
    pub fn apply_all_flag(&mut self) {
        if self.all {
            self.show_correlations = true;
            self.perform_pca = true;
            self.perform_hierarchical_clustering = true;
            self.correlation_network = true;
            self.compute_betweenness = true;
            self.print_summary = true;
            self.time_lag_correlations = true;
            // You might leave outlier handling as is or also enable/disable it if desired.
        }
    }
}
